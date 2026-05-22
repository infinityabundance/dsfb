//! CLI entry point for `dsfb-corpus`.
//!
//! Subcommands (added across the T-section campaign):
//!
//! - `verify` — walk the seed corpus and exit non-zero on any schema
//!   violation. The canonical reproducibility gate for the corpus crate.
//! - `report` — emit the public dedup report (T.6 witness-law sections
//!   and T.7 L-band honesty invariants populated; T.8/T.9 add the
//!   usefulness-ledger and full 10-section report).
//! - `genealogy` — emit the genealogy graph text summary (T.5).
//! - `genealogy-dot` — emit the genealogy graph as Graphviz DOT (T.5).
//! - `genealogy-json` — emit the genealogy graph as JSON (T.5).
//! - `dump` — emit the static seed as TOML (T.2 source-ingestion).
//! - `load-check` — parse a TOML corpus file and assert byte-equivalence
//!   against the static seed (T.2 regression gate).
//!
//! Exit codes:
//!
//! | code | meaning                                              |
//! |------|------------------------------------------------------|
//! |   0  | success                                              |
//! |   1  | usage error / unknown subcommand                     |
//! |   2  | verification failed (schema violations)              |
//! |   5  | I/O failure (cannot write `--out` file)              |
//!
//! No external dependencies; hand-rolled argv parsing matches
//! `dsfb-gpu-debug-demo`'s style.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::process::ExitCode;

use dsfb_gpu_atlas_corpus::activation::{
    collect_activation_plan, render_activation_plan_json, render_activation_plan_text,
    KNOWN_S12_REGISTRY_HASH_V2,
};
use dsfb_gpu_atlas_corpus::activation_audit::{
    build_diff, build_plan_audit, build_transcript_for, render_audit_json, render_audit_text,
    render_diff_json, render_diff_text, render_transcript_json, render_transcript_text,
};
use dsfb_gpu_atlas_corpus::activation_context::{
    build_activation_context, render_activation_context_json, render_activation_context_text,
    render_dataset_manifest_json, render_dataset_manifest_text, render_task_manifest_json,
    render_task_manifest_text, seed_dataset_manifest, seed_task_manifest,
};
use dsfb_gpu_atlas_corpus::admissibility::{
    build_crosswalk, collect_admissibility_grammar, render_crosswalk_json, render_crosswalk_text,
    render_grammar_json, render_grammar_text,
};
use dsfb_gpu_atlas_corpus::amendment::{
    render_amendment_proposal_json, render_amendment_proposal_text, seed_proof_of_life_proposal,
};
use dsfb_gpu_atlas_corpus::challenge_docket::{
    collect_challenge_docket, render_challenge_docket_json, render_challenge_docket_text,
};
use dsfb_gpu_atlas_corpus::consolidate::{
    build_consolidation_report, render_consolidation_report_json, render_consolidation_report_text,
    render_corpus_v2_freeze_json, render_corpus_v2_freeze_text, render_t12_expansion_index_json,
    render_t12_expansion_index_text,
};
use dsfb_gpu_atlas_corpus::contraindication::{
    collect_contraindications, render_contraindications_json, render_contraindications_text,
    render_passport_crosswalk_json, render_passport_crosswalk_text,
};
use dsfb_gpu_atlas_corpus::coverage_holes::{
    collect_coverage_holes, render_coverage_hole_report_json, render_coverage_hole_report_text,
};
use dsfb_gpu_atlas_corpus::dump::dump_to_string;
use dsfb_gpu_atlas_corpus::execution_attestation::{
    build_t11e_live_attestation, render_execution_attestation_json,
    render_execution_attestation_text,
};
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::{
    build_ff1_materialisation_report, build_ff1_passport_index,
    render_ff1_materialisation_report_json, render_ff1_materialisation_report_text,
    render_ff1_passport_index_json, render_ff1_passport_index_text,
};
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate, build_ff2_activation_ratification_gate_summary,
    render_ff2_gate_json, render_ff2_gate_summary_json, render_ff2_gate_summary_text,
    render_ff2_gate_text,
};
use dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::{
    build_ff3_registry_generation_gate, build_ff3_registry_generation_gate_summary,
    render_ff3_gate_json, render_ff3_gate_summary_json, render_ff3_gate_summary_text,
    render_ff3_gate_text,
};
use dsfb_gpu_atlas_corpus::ff4_readme_authority_boundary::{
    build_ff4_readme_authority_boundary_policy, render_ff4_authority_boundary_block,
    render_ff4_policy_json, render_ff4_policy_text,
};
use dsfb_gpu_atlas_corpus::genealogy::{build_genealogy, export_dot, export_json};
use dsfb_gpu_atlas_corpus::loader::load_from_str;
use dsfb_gpu_atlas_corpus::passport::{
    all_passports, passport_for, render_passport_json, render_passport_text,
};
use dsfb_gpu_atlas_corpus::precedent::{
    collect_court_precedents, render_precedents_json, render_precedents_text,
};
use dsfb_gpu_atlas_corpus::proposal_schema_policy::{
    build_proposal_schema_upgrade_policy, render_ff5_migration_table_text, render_ff5_policy_json,
    render_ff5_policy_text,
};
use dsfb_gpu_atlas_corpus::report::{render_genealogy_summary, render_report};
use dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, render_s13d_plan_json, render_s13d_plan_text,
    render_s13d_redundancy_json, render_s13d_redundancy_text, render_s13d_summary_json,
    render_s13d_summary_text,
};
use dsfb_gpu_atlas_corpus::s1_3e_kernel_plan::{
    build_kernel_family_schedule_v1, build_kernel_parameter_table_v1, build_kernel_plan_v1,
    render_kernel_family_schedule_json, render_kernel_family_schedule_text,
    render_kernel_parameter_table_json, render_kernel_parameter_table_text,
    render_kernel_plan_json, render_kernel_plan_text,
};
use dsfb_gpu_atlas_corpus::s1_3f_casefile_v2_activation::{
    build_casefile_v2_authority_chain, render_activation_binding_json,
    render_activation_binding_text, render_authority_chain_json, render_authority_chain_text,
    render_kernel_plan_binding_json, render_kernel_plan_binding_text,
};
use dsfb_gpu_atlas_corpus::s1_3g_otel_binding::{
    build_otel_binding_receipt, render_log_binding_json, render_log_binding_text,
    render_metric_binding_json, render_metric_binding_text, render_otel_binding_receipt_json,
    render_otel_binding_receipt_text, render_resource_binding_json, render_resource_binding_text,
    render_span_binding_json, render_span_binding_text,
};
use dsfb_gpu_atlas_corpus::s_perf_10_digest_lane_plan::{
    render_digest_lane_plan_json, render_digest_lane_plan_text, seed_digest_lane_plan_from_disk,
    verify_digest_lane_plan,
};
use dsfb_gpu_atlas_corpus::s_perf_11_1_post_rewrite_bottleneck_triage::{
    render_post_rewrite_bottleneck_triage_report_json,
    render_post_rewrite_bottleneck_triage_report_text,
    seed_post_rewrite_bottleneck_triage_report_from_disk,
    verify_post_rewrite_bottleneck_triage_report,
};
use dsfb_gpu_atlas_corpus::s_perf_11_measured_digest_compaction::{
    render_bandwidth_delta_report_json, render_bandwidth_delta_report_text,
    seed_bandwidth_delta_report_from_disk, verify_bandwidth_delta_report,
};
use dsfb_gpu_atlas_corpus::s_perf_12_compact_densor_digest_v1_promotion::{
    render_promotion_report_json, render_promotion_report_text,
    seed_s_perf_12_promotion_report_from_disk, verify_promotion_report,
};
use dsfb_gpu_atlas_corpus::s_perf_1_device_traffic_receipt::{
    build_panel_locked_bandwidth_claim_policy, render_bandwidth_claim_policy_json,
    render_bandwidth_claim_policy_text, render_device_traffic_receipt_json,
    render_device_traffic_receipt_text, seed_baseline_uninstrumented_receipt,
};
use dsfb_gpu_atlas_corpus::s_perf_2_layer_a_resident_pipeline::{
    render_layer_a_device_residency_receipt_json, render_layer_a_device_residency_receipt_text,
    render_layer_a_resident_pipeline_json, render_layer_a_resident_pipeline_text,
    render_layer_a_traffic_receipt_json, render_layer_a_traffic_receipt_text,
    seed_baseline_layer_a_pipeline, seed_baseline_layer_a_residency_receipt,
    seed_baseline_layer_a_traffic_receipt,
};
use dsfb_gpu_atlas_corpus::s_perf_3_public_data_saturation_bundle::{
    build_panel_locked_dataset_materialization_policy, render_dataset_materialization_policy_json,
    render_dataset_materialization_policy_text, render_public_data_saturation_bundle_json,
    render_public_data_saturation_bundle_text, seed_baseline_public_data_saturation_bundle,
};
use dsfb_gpu_atlas_corpus::s_perf_4_active_family_compaction::{
    render_active_family_compaction_plan_json, render_active_family_compaction_plan_text,
    render_compacted_parameter_table_receipt_json, render_compacted_parameter_table_receipt_text,
    render_family_compaction_benchmark_schema_json, render_family_compaction_benchmark_schema_text,
    seed_baseline_active_family_compaction_plan, seed_baseline_compacted_parameter_table_receipt,
    seed_baseline_family_compaction_benchmark_schema,
};
use dsfb_gpu_atlas_corpus::s_perf_5_effective_bandwidth_report::{
    render_bandwidth_claim_admission_json, render_bandwidth_claim_admission_text,
    render_effective_bandwidth_report_json, render_effective_bandwidth_report_text,
    render_layer_a_bandwidth_measurement_json, render_layer_a_bandwidth_measurement_text,
    seed_baseline_bandwidth_claim_admission, seed_baseline_effective_bandwidth_report,
    seed_baseline_layer_a_bandwidth_measurement,
};
use dsfb_gpu_atlas_corpus::s_perf_6_rtx4080_super_measured_cuda_pipeline::{
    render_rtx4080_super_measured_bandwidth_claim_json,
    render_rtx4080_super_measured_bandwidth_claim_text,
    render_rtx4080_super_measured_baseline_report_json,
    render_rtx4080_super_measured_baseline_report_text,
    render_rtx4080_super_measured_cuda_pipeline_json,
    render_rtx4080_super_measured_cuda_pipeline_text, seed_rtx4080_super_measured_bandwidth_claim,
    seed_rtx4080_super_measured_baseline_report, seed_rtx4080_super_measured_cuda_pipeline,
};
use dsfb_gpu_atlas_corpus::s_perf_7_source_report_import_verifier::{
    render_source_report_import_verifier_report_json,
    render_source_report_import_verifier_report_text,
    seed_source_report_import_verifier_report_from_disk,
    verify_source_reports_match_s_perf_6_baseline, SeedError,
};
use dsfb_gpu_atlas_corpus::s_perf_8_batched_k_saturation_receipt::{
    render_batched_k_saturation_receipt_json, render_batched_k_saturation_receipt_text,
    seed_batched_k_saturation_receipt_from_disk, verify_batched_k_saturation_receipt,
    SeedError as SPerf8SeedError,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::t12_a_spc::seed_t12_a_spc_proposal;
use dsfb_gpu_atlas_corpus::t12_b_scd::seed_t12_b_scd_proposal;
use dsfb_gpu_atlas_corpus::t12_c_drift::seed_t12_c_drift_proposal;
use dsfb_gpu_atlas_corpus::t12_d_robust::seed_t12_d_robust_proposal;
use dsfb_gpu_atlas_corpus::t12_e_spectral::seed_t12_e_spectral_proposal;
use dsfb_gpu_atlas_corpus::t12_f_timeseries::seed_t12_f_timeseries_proposal;
use dsfb_gpu_atlas_corpus::t12_g_graph::seed_t12_g_graph_proposal;
use dsfb_gpu_atlas_corpus::t12_h_dataquality::seed_t12_h_dataquality_proposal;
use dsfb_gpu_atlas_corpus::t12_i_observability::seed_t12_i_observability_proposal;
use dsfb_gpu_atlas_corpus::t12_j_biosignal::seed_t12_j_biosignal_proposal;
use dsfb_gpu_atlas_corpus::t12_k_industrial::seed_t12_k_industrial_proposal;
use dsfb_gpu_atlas_corpus::t12_l_chemometrics::seed_t12_l_chemometrics_proposal;
use dsfb_gpu_atlas_corpus::t12_m_rf::seed_t12_m_rf_proposal;
use dsfb_gpu_atlas_corpus::t12_n_econometrics_reliability::seed_t12_n_econometrics_reliability_proposal;
use dsfb_gpu_atlas_corpus::t12_o_streaming_sketches::seed_t12_o_streaming_sketches_proposal;
use dsfb_gpu_atlas_corpus::t12_p_information_theory::seed_t12_p_information_theory_proposal;
use dsfb_gpu_atlas_corpus::t12_prov_scientific_provenance::{
    build_provenance_credit_report, build_scientist_credit_index, build_source_bibliography_index,
    render_provenance_credit_report_json, render_provenance_credit_report_text,
    render_scientist_credit_index_json, render_scientist_credit_index_text,
    render_source_bibliography_index_json, render_source_bibliography_index_text,
};
use dsfb_gpu_atlas_corpus::trial_transcript::{
    build_t11d_latency_ramp_fixture, render_trial_transcript_json, render_trial_transcript_text,
};
use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;
use dsfb_gpu_atlas_corpus::verify::verify_corpus;

#[allow(clippy::too_many_lines)]
fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let mut iter = argv.iter().skip(1).cloned();
    let Some(subcommand) = iter.next() else {
        print_usage();
        return ExitCode::from(1);
    };
    let rest: Vec<String> = iter.collect();

    match subcommand.as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "verify" => run_verify(&rest),
        "report" => run_report(&rest),
        "genealogy" => run_genealogy(&rest),
        "dump" => run_dump(&rest),
        "load-check" => run_load_check(&rest),
        "genealogy-dot" => run_genealogy_dot(&rest),
        "genealogy-json" => run_genealogy_json(&rest),
        "report-bundle" => run_report_bundle(&rest),
        "passport" => run_passport(&rest),
        "passports-emit" => run_passports_emit(&rest),
        "precedents" => run_precedents(&rest),
        "precedents-emit" => run_precedents_emit(&rest),
        "admissibility" => run_admissibility(&rest),
        "admissibility-emit" => run_admissibility_emit(&rest),
        "trial-transcript" => run_trial_transcript(&rest),
        "trial-transcript-emit" => run_trial_transcript_emit(&rest),
        "execution-attestation" => run_execution_attestation(&rest),
        "execution-attestation-emit" => run_execution_attestation_emit(&rest),
        "challenges" => run_challenges(&rest),
        "challenges-emit" => run_challenges_emit(&rest),
        "contraindication" => run_contraindication(&rest),
        "contraindications-emit" => run_contraindications_emit(&rest),
        "coverage-holes" => run_coverage_holes(&rest),
        "coverage-holes-emit" => run_coverage_holes_emit(&rest),
        "activation-plan" => run_activation_plan(&rest),
        "activation-plan-emit" => run_activation_plan_emit(&rest),
        "activation-plan-explain" => run_activation_plan_explain(&rest),
        "activation-plan-audit-emit" => run_activation_plan_audit_emit(&rest),
        "activation-plan-diff" => run_activation_plan_diff(&rest),
        "task-manifest" => run_task_manifest(&rest),
        "dataset-manifest" => run_dataset_manifest(&rest),
        "activation-context" => run_activation_context(&rest),
        "activation-context-emit" => run_activation_context_emit(&rest),
        "amendment-proposal" => run_amendment_proposal(&rest),
        "amendment-proposal-emit" => run_amendment_proposal_emit(&rest),
        "t12-a-spc-proposal" => run_t12_a_spc_proposal(&rest),
        "t12-a-spc-proposal-emit" => run_t12_a_spc_proposal_emit(&rest),
        "t12-b-scd-proposal" => run_t12_b_scd_proposal(&rest),
        "t12-b-scd-proposal-emit" => run_t12_b_scd_proposal_emit(&rest),
        "t12-c-drift-proposal" => run_t12_c_drift_proposal(&rest),
        "t12-c-drift-proposal-emit" => run_t12_c_drift_proposal_emit(&rest),
        "t12-d-robust-proposal" => run_t12_d_robust_proposal(&rest),
        "t12-d-robust-proposal-emit" => run_t12_d_robust_proposal_emit(&rest),
        "t12-e-spectral-proposal" => run_t12_e_spectral_proposal(&rest),
        "t12-e-spectral-proposal-emit" => run_t12_e_spectral_proposal_emit(&rest),
        "t12-f-timeseries-proposal" => run_t12_f_timeseries_proposal(&rest),
        "t12-f-timeseries-proposal-emit" => run_t12_f_timeseries_proposal_emit(&rest),
        "t12-g-graph-proposal" => run_t12_g_graph_proposal(&rest),
        "t12-g-graph-proposal-emit" => run_t12_g_graph_proposal_emit(&rest),
        "t12-h-dataquality-proposal" => run_t12_h_dataquality_proposal(&rest),
        "t12-h-dataquality-proposal-emit" => run_t12_h_dataquality_proposal_emit(&rest),
        "t12-i-observability-proposal" => run_t12_i_observability_proposal(&rest),
        "t12-i-observability-proposal-emit" => run_t12_i_observability_proposal_emit(&rest),
        "t12-j-biosignal-proposal" => run_t12_j_biosignal_proposal(&rest),
        "t12-j-biosignal-proposal-emit" => run_t12_j_biosignal_proposal_emit(&rest),
        "t12-k-industrial-proposal" => run_t12_k_industrial_proposal(&rest),
        "t12-k-industrial-proposal-emit" => run_t12_k_industrial_proposal_emit(&rest),
        "t12-l-chemometrics-proposal" => run_t12_l_chemometrics_proposal(&rest),
        "t12-l-chemometrics-proposal-emit" => run_t12_l_chemometrics_proposal_emit(&rest),
        "t12-m-rf-proposal" => run_t12_m_rf_proposal(&rest),
        "t12-m-rf-proposal-emit" => run_t12_m_rf_proposal_emit(&rest),
        "t12-n-econometrics-reliability-proposal" => {
            run_t12_n_econometrics_reliability_proposal(&rest)
        }
        "t12-n-econometrics-reliability-proposal-emit" => {
            run_t12_n_econometrics_reliability_proposal_emit(&rest)
        }
        "t12-o-streaming-sketches-proposal" => run_t12_o_streaming_sketches_proposal(&rest),
        "t12-o-streaming-sketches-proposal-emit" => {
            run_t12_o_streaming_sketches_proposal_emit(&rest)
        }
        "t12-p-information-theory-proposal" => run_t12_p_information_theory_proposal(&rest),
        "t12-p-information-theory-proposal-emit" => {
            run_t12_p_information_theory_proposal_emit(&rest)
        }
        "t12-consolidate-report" => run_t12_consolidate_report(&rest),
        "t12-consolidate-report-emit" => run_t12_consolidate_report_emit(&rest),
        "t12-corpus-v2-freeze" => run_t12_corpus_v2_freeze(&rest),
        "t12-corpus-v2-freeze-emit" => run_t12_corpus_v2_freeze_emit(&rest),
        "t12-expansion-index" => run_t12_expansion_index(&rest),
        "t12-expansion-index-emit" => run_t12_expansion_index_emit(&rest),
        "ff1-passport-index" => run_ff1_passport_index(&rest),
        "ff1-passport-index-emit" => run_ff1_passport_index_emit(&rest),
        "ff1-materialisation-report" => run_ff1_materialisation_report(&rest),
        "ff1-materialisation-report-emit" => run_ff1_materialisation_report_emit(&rest),
        "ff2-gate" => run_ff2_gate(&rest),
        "ff2-gate-emit" => run_ff2_gate_emit(&rest),
        "ff2-gate-summary" => run_ff2_gate_summary(&rest),
        "ff2-gate-summary-emit" => run_ff2_gate_summary_emit(&rest),
        "ff3-gate" => run_ff3_gate(&rest),
        "ff3-gate-emit" => run_ff3_gate_emit(&rest),
        "ff3-gate-summary" => run_ff3_gate_summary(&rest),
        "ff3-gate-summary-emit" => run_ff3_gate_summary_emit(&rest),
        "ff4-policy" => run_ff4_policy(&rest),
        "ff4-policy-emit" => run_ff4_policy_emit(&rest),
        "ff4-authority-boundary-block" => run_ff4_authority_boundary_block(&rest),
        "ff5-policy" => run_ff5_policy(&rest),
        "ff5-policy-emit" => run_ff5_policy_emit(&rest),
        "ff5-migration-table" => run_ff5_migration_table(&rest),
        "s1-3d-plan" => run_s1_3d_plan(&rest),
        "s1-3d-plan-emit" => run_s1_3d_plan_emit(&rest),
        "s1-3d-redundancy" => run_s1_3d_redundancy(&rest),
        "s1-3d-summary" => run_s1_3d_summary(&rest),
        "s1-3e-plan" => run_s1_3e_plan(&rest),
        "s1-3e-plan-emit" => run_s1_3e_plan_emit(&rest),
        "s1-3e-schedule" => run_s1_3e_schedule(&rest),
        "s1-3e-parameter-table" => run_s1_3e_parameter_table(&rest),
        "s1-3f-authority-chain" => run_s1_3f_authority_chain(&rest),
        "s1-3f-authority-chain-emit" => run_s1_3f_authority_chain_emit(&rest),
        "s1-3f-activation-binding" => run_s1_3f_activation_binding(&rest),
        "s1-3f-kernel-plan-binding" => run_s1_3f_kernel_plan_binding(&rest),
        "s1-3g-binding" => run_s1_3g_binding(&rest),
        "s1-3g-binding-emit" => run_s1_3g_binding_emit(&rest),
        "s1-3g-span-binding" => run_s1_3g_span_binding(&rest),
        "s1-3g-metric-binding" => run_s1_3g_metric_binding(&rest),
        "s1-3g-log-binding" => run_s1_3g_log_binding(&rest),
        "s1-3g-resource-binding" => run_s1_3g_resource_binding(&rest),
        "t12-prov-report" => run_t12_prov_report(&rest),
        "t12-prov-report-emit" => run_t12_prov_report_emit(&rest),
        "t12-prov-scientist-credit-index" => run_t12_prov_scientist_credit_index(&rest),
        "t12-prov-source-bibliography-index" => run_t12_prov_source_bibliography_index(&rest),
        "s-perf-1-receipt" => run_s_perf_1_receipt(&rest),
        "s-perf-1-receipt-emit" => run_s_perf_1_receipt_emit(&rest),
        "s-perf-1-policy" => run_s_perf_1_policy(&rest),
        "s-perf-1-policy-emit" => run_s_perf_1_policy_emit(&rest),
        "s-perf-2-pipeline" => run_s_perf_2_pipeline(&rest),
        "s-perf-2-residency-receipt" => run_s_perf_2_residency_receipt(&rest),
        "s-perf-2-traffic-receipt" => run_s_perf_2_traffic_receipt(&rest),
        "s-perf-2-receipts-emit" => run_s_perf_2_receipts_emit(&rest),
        "s-perf-3-bundle" => run_s_perf_3_bundle(&rest),
        "s-perf-3-policy" => run_s_perf_3_policy(&rest),
        "s-perf-3-bundle-emit" => run_s_perf_3_bundle_emit(&rest),
        "s-perf-4-plan" => run_s_perf_4_plan(&rest),
        "s-perf-4-parameter-table-receipt" => run_s_perf_4_parameter_table_receipt(&rest),
        "s-perf-4-schema" => run_s_perf_4_schema(&rest),
        "s-perf-4-receipts-emit" => run_s_perf_4_receipts_emit(&rest),
        "s-perf-5-measurement" => run_s_perf_5_measurement(&rest),
        "s-perf-5-admission" => run_s_perf_5_admission(&rest),
        "s-perf-5-report" => run_s_perf_5_report(&rest),
        "s-perf-5-receipts-emit" => run_s_perf_5_receipts_emit(&rest),
        "s-perf-6-measurement" => run_s_perf_6_measurement(&rest),
        "s-perf-6-claim" => run_s_perf_6_claim(&rest),
        "s-perf-6-baseline" => run_s_perf_6_baseline(&rest),
        "s-perf-6-receipts-emit" => run_s_perf_6_receipts_emit(&rest),
        "s-perf-7-verifier" => run_s_perf_7_verifier(&rest),
        "s-perf-7-verifier-emit" => run_s_perf_7_verifier_emit(&rest),
        "s-perf-8-batched-k" => run_s_perf_8_batched_k(&rest),
        "s-perf-8-batched-k-emit" => run_s_perf_8_batched_k_emit(&rest),
        "s-perf-10-digest-lane" => run_s_perf_10_digest_lane(&rest),
        "s-perf-10-digest-lane-emit" => run_s_perf_10_digest_lane_emit(&rest),
        "s-perf-11-digest-compaction" => run_s_perf_11_digest_compaction(&rest),
        "s-perf-11-digest-compaction-emit" => run_s_perf_11_digest_compaction_emit(&rest),
        "s-perf-11-1-bottleneck-triage" => run_s_perf_11_1_bottleneck_triage(&rest),
        "s-perf-11-1-bottleneck-triage-emit" => run_s_perf_11_1_bottleneck_triage_emit(&rest),
        "s-perf-12-promotion" => run_s_perf_12_promotion(&rest),
        "s-perf-12-promotion-emit" => run_s_perf_12_promotion_emit(&rest),
        other => {
            eprintln!("dsfb-corpus: unknown subcommand `{other}`");
            print_usage();
            ExitCode::from(1)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn print_usage() {
    eprintln!(
        "dsfb-corpus: literature detector corpus + canonicalisation court

Usage:
  dsfb-corpus verify
  dsfb-corpus report          [--out PATH]
  dsfb-corpus genealogy       [--out PATH]
  dsfb-corpus genealogy-dot   [--out PATH]
  dsfb-corpus genealogy-json  [--out PATH]
  dsfb-corpus dump            [--out PATH]
  dsfb-corpus load-check      [--from PATH]
  dsfb-corpus report-bundle   [--out-dir DIR]
  dsfb-corpus passport <id>   [--json]
  dsfb-corpus passports-emit  [--out-dir DIR]
  dsfb-corpus precedents       [--json] [--out PATH]
  dsfb-corpus precedents-emit  [--out-dir DIR]
  dsfb-corpus admissibility       [--json] [--out PATH]
  dsfb-corpus admissibility-emit  [--out-dir DIR]
  dsfb-corpus trial-transcript        [--json] [--out PATH]
  dsfb-corpus trial-transcript-emit   [--out-dir DIR]
  dsfb-corpus execution-attestation   [--json] [--out PATH]
  dsfb-corpus execution-attestation-emit [--out-dir DIR]
  dsfb-corpus challenges              [--json] [--out PATH]
  dsfb-corpus challenges-emit         [--out-dir DIR]
  dsfb-corpus contraindication        [--json] [--out PATH]
  dsfb-corpus contraindications-emit  [--out-dir DIR]
  dsfb-corpus coverage-holes          [--json] [--out PATH]
  dsfb-corpus coverage-holes-emit     [--out-dir DIR]
  dsfb-corpus activation-plan         [--json] [--out PATH] [--registry-hash HEX]
  dsfb-corpus activation-plan-emit    [--out-dir DIR]       [--registry-hash HEX]
  dsfb-corpus activation-plan-explain <canonical_id> [--json] [--out PATH]
  dsfb-corpus activation-plan-audit-emit [--out-dir DIR]
  dsfb-corpus activation-plan-diff    --old OLD.json --new NEW.json [--json] [--out PATH]
  dsfb-corpus task-manifest           [--json] [--out PATH]
  dsfb-corpus dataset-manifest        [--json] [--out PATH]
  dsfb-corpus activation-context      [--json] [--out PATH]
  dsfb-corpus activation-context-emit [--out-dir DIR]
  dsfb-corpus amendment-proposal      [--json] [--out PATH]
  dsfb-corpus amendment-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-a-spc-proposal      [--json] [--out PATH]
  dsfb-corpus t12-a-spc-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-b-scd-proposal      [--json] [--out PATH]
  dsfb-corpus t12-b-scd-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-c-drift-proposal      [--json] [--out PATH]
  dsfb-corpus t12-c-drift-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-d-robust-proposal      [--json] [--out PATH]
  dsfb-corpus t12-d-robust-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-e-spectral-proposal      [--json] [--out PATH]
  dsfb-corpus t12-e-spectral-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-f-timeseries-proposal      [--json] [--out PATH]
  dsfb-corpus t12-f-timeseries-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-g-graph-proposal      [--json] [--out PATH]
  dsfb-corpus t12-g-graph-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-h-dataquality-proposal      [--json] [--out PATH]
  dsfb-corpus t12-h-dataquality-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-i-observability-proposal      [--json] [--out PATH]
  dsfb-corpus t12-i-observability-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-j-biosignal-proposal           [--json] [--out PATH]
  dsfb-corpus t12-j-biosignal-proposal-emit      [--out-dir DIR]
  dsfb-corpus t12-k-industrial-proposal          [--json] [--out PATH]
  dsfb-corpus t12-k-industrial-proposal-emit     [--out-dir DIR]
  dsfb-corpus t12-l-chemometrics-proposal        [--json] [--out PATH]
  dsfb-corpus t12-l-chemometrics-proposal-emit   [--out-dir DIR]
  dsfb-corpus t12-m-rf-proposal                  [--json] [--out PATH]
  dsfb-corpus t12-m-rf-proposal-emit             [--out-dir DIR]
  dsfb-corpus t12-n-econometrics-reliability-proposal      [--json] [--out PATH]
  dsfb-corpus t12-n-econometrics-reliability-proposal-emit [--out-dir DIR]
  dsfb-corpus t12-o-streaming-sketches-proposal            [--json] [--out PATH]
  dsfb-corpus t12-o-streaming-sketches-proposal-emit       [--out-dir DIR]
  dsfb-corpus t12-p-information-theory-proposal            [--json] [--out PATH]
  dsfb-corpus t12-p-information-theory-proposal-emit       [--out-dir DIR]
  dsfb-corpus t12-consolidate-report                       [--json] [--out PATH]
  dsfb-corpus t12-consolidate-report-emit                  [--out-dir DIR]
  dsfb-corpus t12-corpus-v2-freeze                         [--json] [--out PATH]
  dsfb-corpus t12-corpus-v2-freeze-emit                    [--out-dir DIR]
  dsfb-corpus t12-expansion-index                          [--json] [--out PATH]
  dsfb-corpus t12-expansion-index-emit                     [--out-dir DIR]
  dsfb-corpus ff1-passport-index                           [--json] [--out PATH]
  dsfb-corpus ff1-passport-index-emit                      [--out-dir DIR]
  dsfb-corpus ff1-materialisation-report                   [--json] [--out PATH]
  dsfb-corpus ff1-materialisation-report-emit              [--out-dir DIR]
  dsfb-corpus ff2-gate                                     [--json] [--out PATH]
  dsfb-corpus ff2-gate-emit                                [--out-dir DIR]
  dsfb-corpus ff2-gate-summary                             [--json] [--out PATH]
  dsfb-corpus ff2-gate-summary-emit                        [--out-dir DIR]
  dsfb-corpus ff3-gate                                     [--json] [--out PATH]
  dsfb-corpus ff3-gate-emit                                [--out-dir DIR]
  dsfb-corpus ff3-gate-summary                             [--json] [--out PATH]
  dsfb-corpus ff3-gate-summary-emit                        [--out-dir DIR]
  dsfb-corpus ff4-policy                                   [--json] [--out PATH]
  dsfb-corpus ff4-policy-emit                              [--out-dir DIR]
  dsfb-corpus ff4-authority-boundary-block                 [--out PATH]
  dsfb-corpus ff5-policy                                   [--json] [--out PATH]
  dsfb-corpus ff5-policy-emit                              [--out-dir DIR]
  dsfb-corpus ff5-migration-table                          [--out PATH]
  dsfb-corpus s1-3d-plan                                   [--json] [--out PATH]
  dsfb-corpus s1-3d-plan-emit                              [--out-dir DIR]
  dsfb-corpus s1-3d-redundancy                             [--json] [--out PATH]
  dsfb-corpus s1-3d-summary                                [--json] [--out PATH]
  dsfb-corpus s1-3e-plan                                   [--json] [--out PATH]
  dsfb-corpus s1-3e-plan-emit                              [--out-dir DIR]
  dsfb-corpus s1-3e-schedule                               [--json] [--out PATH]
  dsfb-corpus s1-3e-parameter-table                        [--json] [--out PATH]
  dsfb-corpus s1-3f-authority-chain                        [--json] [--out PATH]
  dsfb-corpus s1-3f-authority-chain-emit                   [--out-dir DIR]
  dsfb-corpus s1-3f-activation-binding                     [--json] [--out PATH]
  dsfb-corpus s1-3f-kernel-plan-binding                    [--json] [--out PATH]
  dsfb-corpus s1-3g-binding                                [--json] [--out PATH]
  dsfb-corpus s1-3g-binding-emit                           [--out-dir DIR]
  dsfb-corpus s1-3g-span-binding                           [--json] [--out PATH]
  dsfb-corpus s1-3g-metric-binding                         [--json] [--out PATH]
  dsfb-corpus s1-3g-log-binding                            [--json] [--out PATH]
  dsfb-corpus s1-3g-resource-binding                       [--json] [--out PATH]
  dsfb-corpus t12-prov-report                              [--json] [--out PATH]
  dsfb-corpus t12-prov-report-emit                         [--out-dir DIR]
  dsfb-corpus t12-prov-scientist-credit-index              [--json] [--out PATH]
  dsfb-corpus t12-prov-source-bibliography-index           [--json] [--out PATH]
  dsfb-corpus s-perf-1-receipt                             [--json] [--out PATH]
  dsfb-corpus s-perf-1-receipt-emit                        [--out-dir DIR]
  dsfb-corpus s-perf-1-policy                              [--json] [--out PATH]
  dsfb-corpus s-perf-1-policy-emit                         [--out-dir DIR]
  dsfb-corpus s-perf-2-pipeline                            [--json] [--out PATH]
  dsfb-corpus s-perf-2-residency-receipt                   [--json] [--out PATH]
  dsfb-corpus s-perf-2-traffic-receipt                     [--json] [--out PATH]
  dsfb-corpus s-perf-2-receipts-emit                       [--out-dir DIR]
  dsfb-corpus s-perf-3-bundle                              [--json] [--out PATH]
  dsfb-corpus s-perf-3-policy                              [--json] [--out PATH]
  dsfb-corpus s-perf-3-bundle-emit                         [--out-dir DIR]
  dsfb-corpus s-perf-4-plan                                [--json] [--out PATH]
  dsfb-corpus s-perf-4-parameter-table-receipt             [--json] [--out PATH]
  dsfb-corpus s-perf-4-schema                              [--json] [--out PATH]
  dsfb-corpus s-perf-4-receipts-emit                       [--out-dir DIR]
  dsfb-corpus s-perf-5-measurement                         [--json] [--out PATH]
  dsfb-corpus s-perf-5-admission                           [--json] [--out PATH]
  dsfb-corpus s-perf-5-report                              [--json] [--out PATH]
  dsfb-corpus s-perf-5-receipts-emit                       [--out-dir DIR]
  dsfb-corpus s-perf-6-measurement                         [--json] [--out PATH]
  dsfb-corpus s-perf-6-claim                               [--json] [--out PATH]
  dsfb-corpus s-perf-6-baseline                            [--json] [--out PATH]
  dsfb-corpus s-perf-6-receipts-emit                       [--out-dir DIR]
  dsfb-corpus s-perf-7-verifier                            [--json] [--out PATH]
  dsfb-corpus s-perf-7-verifier-emit                       [--out-dir DIR]
  dsfb-corpus s-perf-8-batched-k                           [--json] [--out PATH]
  dsfb-corpus s-perf-8-batched-k-emit                      [--out-dir DIR]
  dsfb-corpus s-perf-10-digest-lane                        [--json] [--out PATH]
  dsfb-corpus s-perf-10-digest-lane-emit                   [--out-dir DIR]
  dsfb-corpus s-perf-11-digest-compaction                  [--json] [--out PATH]
  dsfb-corpus s-perf-11-digest-compaction-emit             [--out-dir DIR]
  dsfb-corpus s-perf-11-1-bottleneck-triage                [--json] [--out PATH]
  dsfb-corpus s-perf-11-1-bottleneck-triage-emit           [--out-dir DIR]
  dsfb-corpus s-perf-12-promotion                          [--json] [--out PATH]
  dsfb-corpus s-perf-12-promotion-emit                     [--out-dir DIR]

Subcommands:
  verify          Walk the seed corpus and exit 0 if every record passes
                  the schema invariants. Exits 2 on any violation, with a
                  per-record diagnostic to stderr.
  report          Emit the public dedup-report (T.6 witness-law sections,
                  T.7 L-band honesty invariants, T.8 usefulness-ledger
                  honesty invariants populated; T.9 emits the internal
                  audit bundle via `report-bundle`).
  genealogy       Emit the genealogy graph text summary (T.5).
  genealogy-dot   Emit the genealogy graph as Graphviz DOT (T.5).
  genealogy-json  Emit the genealogy graph as JSON (T.5).
  dump            Emit the static seed as TOML (T.2 source-ingestion
                  format). Use `--out PATH` to write to a file (commonly
                  `corpus/corpus.toml`).
  load-check      Parse a TOML corpus file and assert byte-equivalence
                  against the static seed. Exits 2 on any divergence,
                  with per-record diagnostics to stderr. Path defaults
                  to `corpus/corpus.toml` when no `--from` flag is given.
  report-bundle   Emit the T.9 internal corpus audit bundle:
                  corpus_t9_audit_report.txt / .json plus refreshed
                  corpus_t9_genealogy.dot / .json. INTERNAL audit only;
                  not a publication artifact. corpus_hash_v1 and
                  CaseFileV2 are deferred. Default --out-dir is
                  `reports/`.
  passport        T.11a — emit one DetectorPassport for the given
                  canonical id. Renders text by default; pass `--json`
                  for the deterministic JSON object form. The passport
                  bundles every T.1-T.10 fact (identity hashes, dedup
                  decision, genealogy edges, witness role, fusion
                  planes, L-band, usefulness evidence level, lifecycle,
                  constitution flags) into one hashable record.
  passports-emit  T.11a — bulk emit every SEED passport to
                  `<out-dir>/passports.txt` + `<out-dir>/passports.json`.
                  Default `--out-dir` is
                  `crates/dsfb-gpu-atlas-corpus/out/`. Two invocations
                  produce byte-identical files.
  precedents      T.11b — emit the full court-precedent ledger.
                  Renders text by default; pass `--json` for the
                  deterministic JSON object form. The precedent
                  ledger is derived from T.4 / T.6 / T.7 / T.8 /
                  T.10 / S1.2 / T.11a / T.9 deferred gates and
                  carries `precedent_hash_v1`.
  precedents-emit T.11b — bulk emit the precedent ledger to
                  `<out-dir>/court_precedents.txt` +
                  `<out-dir>/court_precedents.json`. Default
                  `--out-dir` is `crates/dsfb-gpu-atlas-corpus/out/`.
                  Two invocations produce byte-identical files.
  admissibility   T.11c — emit the admissibility-grammar
                  snapshot: episode-admissibility + confuser-
                  suppression rules, each citing T.11b
                  precedents. Carries `admissibility_grammar_hash_v1`.
                  Renders text by default; `--json` switches to
                  the deterministic JSON object form.
  admissibility-emit
                  T.11c — bulk emit four artifacts to
                  `<out-dir>/`: `admissibility_grammar.txt`,
                  `admissibility_grammar.json`,
                  `passport_grammar_crosswalk.txt`,
                  `passport_grammar_crosswalk.json`. Two
                  invocations produce byte-identical files.
  trial-transcript
                  T.11d — emit the panel-locked synthetic
                  LatencyRamp trial transcript fixture. Renders
                  text by default; `--json` switches to the
                  deterministic JSON form. Carries
                  `trial_transcript_hash_v1`. NOT yet derived
                  from GPU-produced CaseFileV1 episodes.
  trial-transcript-emit
                  T.11d — bulk emit
                  `<out-dir>/trial_transcript_v1.txt` and
                  `<out-dir>/trial_transcript_v1.json`. Two
                  invocations produce byte-identical files.
  execution-attestation
                  T.11e — emit the unsigned DSFB-native local
                  execution-attestation receipt (SLSA / in-toto-
                  inspired shape, NOT a SLSA compliance claim,
                  NOT an in-toto signed statement). Queries the
                  live environment (git commit, rustc / cargo /
                  nvcc versions, ...) and binds every hash-chain
                  anchor (corpus / registry / precedent / grammar
                  / trial-transcript). Carries `receipt_hash_v1`.
                  Renders text by default; pass `--json` for the
                  deterministic JSON form.
  execution-attestation-emit
                  T.11e — bulk emit
                  `<out-dir>/execution_attestation_v1.txt` and
                  `<out-dir>/execution_attestation_v1.json`. The
                  receipt is **unsigned**.
  challenges      T.11f — render the ChallengeDocketV1 adversarial
                  overlay: 10-entry conservative seed of honest
                  challenges across detector / precedent / grammar
                  / transcript / receipt targets, each carrying a
                  status (Open / Sustained / Overruled / Deferred
                  / Superseded), severity, evidence pointers, and
                  proposed-resolution / court-response pairs. The
                  docket does NOT mutate the corpus; sustaining a
                  challenge requires a separate later commit.
                  Carries `challenge_docket_hash_v1`. Renders text
                  by default; pass `--json` for deterministic JSON.
  challenges-emit T.11f — bulk emit
                  `<out-dir>/challenge_docket_v1.txt` and
                  `<out-dir>/challenge_docket_v1.json`. Two
                  invocations produce byte-identical files.
  contraindication
                  T.11g — emit the
                  DetectorContraindicationReceiptV1 snapshot:
                  per-detector contraindications (works-best-when,
                  fails-when, known-confusers, required-sampling-
                  law, required-units, minimum-support,
                  do-not-use-for, closest-aliases, closest-non-
                  aliases) + adversarial-twin layer. Renders text
                  by default; pass `--json` for the deterministic
                  JSON form. Carries
                  `detector_contraindication_hash_v1`.
  contraindications-emit
                  T.11g — bulk emit four artifacts:
                  `<out-dir>/contraindications_v1.{{txt,json}}`
                  (the receipts) and
                  `<out-dir>/passport_contraindication_crosswalk.{{txt,json}}`
                  (the per-canonical-id passport linkage; lives
                  in a separate namespace so passport hashes do
                  not churn). Two invocations produce byte-
                  identical files.
  coverage-holes  T.11h — emit the CoverageHoleReportV1
                  snapshot: per-detector / per-family / per-
                  surface coverage gaps aggregated across the
                  seven panel-locked categories (detector,
                  witness-law, implementation, semantics,
                  jurisprudence, source/provenance, reason-code).
                  Headline metric is per-surface Reason-Code
                  Coverage. Renders text by default; pass
                  `--json` for the deterministic JSON form.
                  Carries `coverage_hole_hash_v1`. AUDIT
                  surface only — does NOT mutate any upstream
                  hash or repair any hole.
  coverage-holes-emit
                  T.11h — bulk emit two artifacts:
                  `<out-dir>/coverage_holes_v1.{{txt,json}}`.
                  Two invocations produce byte-identical files.
  activation-plan S1.3a — emit the ActivationPlanV1 snapshot:
                  per-detector reason-coded enable / disable /
                  warn-only / deferred decisions consuming the
                  sealed T.11 court stack (passport,
                  contraindication, challenge-docket, coverage-
                  hole) plus `corpus_hash_v1` (T.10) and
                  `registry_hash_v2` (S1.2). Renders text by
                  default; pass `--json` for the JSON form.
                  `--registry-hash <hex>` overrides the pinned
                  `KNOWN_S12_REGISTRY_HASH_V2` constant when a
                  caller wants to plan against a different
                  registry. Carries `activation_plan_hash_v1`
                  in the new namespace
                  `DSFB-GPU-ATLAS:ACTIVATION-PLAN:v1\\0`. Does
                  NOT mutate any upstream hash anchor.
  activation-plan-emit
                  S1.3a — bulk emit two artifacts:
                  `<out-dir>/activation_plan_v1.{{txt,json}}`.
                  Two invocations against the same court stack
                  produce byte-identical files.

Common flags:
  --out PATH   Write subcommand output to PATH instead of stdout.
               If the file already exists it is overwritten.
  --from PATH  Read a TOML corpus from PATH (for `load-check`)."
    );
}

fn run_verify(_rest: &[String]) -> ExitCode {
    let report = verify_corpus(SEED);
    println!("verify: inspected {} records", report.records_inspected);
    if report.is_clean() {
        println!("verify: clean ({} records pass)", report.records_inspected);
        ExitCode::SUCCESS
    } else {
        let failed = report.unique_failed_records();
        eprintln!(
            "verify: FAILED ({} errors across {} records)",
            report.errors.len(),
            failed
        );
        for err in &report.errors {
            eprintln!("  [{:>3}] {}", err.canonical_id.0, err.kind.describe());
        }
        ExitCode::from(2)
    }
}

fn run_report(rest: &[String]) -> ExitCode {
    let body = render_report(SEED);
    write_or_print(rest, &body)
}

fn run_genealogy(rest: &[String]) -> ExitCode {
    let body = render_genealogy_summary(SEED);
    write_or_print(rest, &body)
}

fn run_dump(rest: &[String]) -> ExitCode {
    let body = dump_to_string(SEED);
    write_or_print(rest, &body)
}

fn run_genealogy_dot(rest: &[String]) -> ExitCode {
    let graph = build_genealogy();
    let body = export_dot(&graph);
    write_or_print(rest, &body)
}

fn run_genealogy_json(rest: &[String]) -> ExitCode {
    let graph = build_genealogy();
    let body = export_json(&graph);
    write_or_print(rest, &body)
}

fn run_load_check(rest: &[String]) -> ExitCode {
    let mut from_path: Option<&str> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        if arg == "--from" {
            if idx + 1 >= rest.len() {
                eprintln!("dsfb-corpus: `--from` requires a path argument");
                return ExitCode::from(1);
            }
            from_path = Some(rest[idx + 1].as_str());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--from=") {
            from_path = Some(value);
            idx += 1;
        } else {
            eprintln!("dsfb-corpus: unknown flag `{arg}`");
            return ExitCode::from(1);
        }
    }
    let path = from_path.unwrap_or("corpus/corpus.toml");
    let bytes = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("dsfb-corpus: failed to read `{path}`: {err}");
            return ExitCode::from(5);
        }
    };
    let loaded = match load_from_str(&bytes) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("dsfb-corpus: load failed: {}", err.display());
            return ExitCode::from(2);
        }
    };
    if loaded.len() != SEED.len() {
        eprintln!(
            "dsfb-corpus: record-count mismatch: TOML has {}, static seed has {}",
            loaded.len(),
            SEED.len()
        );
        return ExitCode::from(2);
    }
    let mut mismatches = 0usize;
    for (l, s) in loaded.iter().zip(SEED.iter()) {
        if !l.matches_static(s) {
            mismatches += 1;
            eprintln!(
                "dsfb-corpus: divergence at canonical_id {} (`{}`)",
                s.canonical_id.0, s.display_name
            );
        }
    }
    if mismatches > 0 {
        eprintln!("dsfb-corpus: load-check FAILED ({mismatches} record(s) diverge)");
        return ExitCode::from(2);
    }
    println!(
        "load-check: clean ({} records loaded; all match the static seed)",
        loaded.len()
    );
    ExitCode::SUCCESS
}

fn run_report_bundle(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "reports";
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        if arg == "--out-dir" {
            if idx + 1 >= rest.len() {
                eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                return ExitCode::from(1);
            }
            out_dir = rest[idx + 1].as_str();
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--out-dir=") {
            out_dir = value;
            idx += 1;
        } else {
            eprintln!("dsfb-corpus: unknown flag `{arg}`");
            return ExitCode::from(1);
        }
    }
    let bundle = dsfb_gpu_atlas_corpus::audit_report::generate_audit_report_bundle();
    let files: &[(&str, &str)] = &[
        ("corpus_t9_audit_report.txt", &bundle.audit_report_txt),
        ("corpus_t9_audit_report.json", &bundle.audit_report_json),
        ("corpus_t9_genealogy.dot", &bundle.genealogy_dot),
        ("corpus_t9_genealogy.json", &bundle.genealogy_json),
    ];
    for (name, body) in files {
        let path = format!("{out_dir}/{name}");
        if let Err(err) = std::fs::write(&path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path}");
    }
    ExitCode::SUCCESS
}

fn write_or_print(rest: &[String], body: &str) -> ExitCode {
    let mut out_path: Option<&str> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        if arg == "--out" {
            if idx + 1 >= rest.len() {
                eprintln!("dsfb-corpus: `--out` requires a path argument");
                return ExitCode::from(1);
            }
            out_path = Some(rest[idx + 1].as_str());
            idx += 2;
        } else if let Some(value) = arg.strip_prefix("--out=") {
            out_path = Some(value);
            idx += 1;
        } else {
            eprintln!("dsfb-corpus: unknown flag `{arg}`");
            return ExitCode::from(1);
        }
    }
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(path, body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_passport(rest: &[String]) -> ExitCode {
    if rest.is_empty() {
        eprintln!("dsfb-corpus: `passport` requires a canonical-id argument");
        return ExitCode::from(1);
    }
    let id_arg = &rest[0];
    let Ok(id) = id_arg.parse::<u32>() else {
        eprintln!("dsfb-corpus: `passport` expected a u32 canonical id, got `{id_arg}`");
        return ExitCode::from(1);
    };
    let mut json_mode = false;
    let mut idx = 1;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `passport`");
                return ExitCode::from(1);
            }
        }
    }
    let Some(p) = passport_for(DetectorCanonicalId(id)) else {
        eprintln!("dsfb-corpus: no passport for canonical id {id}");
        return ExitCode::from(2);
    };
    let body = if json_mode {
        render_passport_json(&p)
    } else {
        render_passport_text(&p)
    };
    print!("{body}");
    ExitCode::SUCCESS
}

fn run_passports_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `passports-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let passports = all_passports();
    let mut text = String::new();
    let mut json = String::from("[\n");
    for (i, p) in passports.iter().enumerate() {
        text.push_str(&render_passport_text(p));
        text.push('\n');
        json.push_str(&render_passport_json(p));
        if i + 1 < passports.len() {
            // Strip trailing newline from the rendered JSON and
            // join with a comma so the result is a single JSON
            // array.
            if json.ends_with('\n') {
                json.pop();
            }
            json.push(',');
            json.push('\n');
        }
    }
    if json.ends_with('\n') {
        json.pop();
    }
    json.push_str("\n]\n");
    let text_path = format!("{out_dir}/passports.txt");
    let json_path = format!("{out_dir}/passports.json");
    if let Err(err) = std::fs::write(&text_path, &text) {
        eprintln!("dsfb-corpus: failed to write `{text_path}`: {err}");
        return ExitCode::from(5);
    }
    if let Err(err) = std::fs::write(&json_path, &json) {
        eprintln!("dsfb-corpus: failed to write `{json_path}`: {err}");
        return ExitCode::from(5);
    }
    println!("wrote {text_path} ({} bytes)", text.len());
    println!("wrote {json_path} ({} bytes)", json.len());
    ExitCode::SUCCESS
}

fn run_precedents(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `precedents`");
                return ExitCode::from(1);
            }
        }
    }
    let set = collect_court_precedents();
    let body = if json_mode {
        render_precedents_json(&set)
    } else {
        render_precedents_text(&set)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_precedents_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `precedents-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let set = collect_court_precedents();
    let text = render_precedents_text(&set);
    let json = render_precedents_json(&set);
    let text_path = format!("{out_dir}/court_precedents.txt");
    let json_path = format!("{out_dir}/court_precedents.json");
    if let Err(err) = std::fs::write(&text_path, &text) {
        eprintln!("dsfb-corpus: failed to write `{text_path}`: {err}");
        return ExitCode::from(5);
    }
    if let Err(err) = std::fs::write(&json_path, &json) {
        eprintln!("dsfb-corpus: failed to write `{json_path}`: {err}");
        return ExitCode::from(5);
    }
    println!("wrote {text_path} ({} bytes)", text.len());
    println!("wrote {json_path} ({} bytes)", json.len());
    ExitCode::SUCCESS
}

fn run_admissibility(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `admissibility`");
                return ExitCode::from(1);
            }
        }
    }
    let snapshot = collect_admissibility_grammar();
    let body = if json_mode {
        render_grammar_json(&snapshot)
    } else {
        render_grammar_text(&snapshot)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_admissibility_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `admissibility-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let snapshot = collect_admissibility_grammar();
    let snapshot_text = render_grammar_text(&snapshot);
    let snapshot_json = render_grammar_json(&snapshot);
    let crosswalk = build_crosswalk(&snapshot);
    let crosswalk_text = render_crosswalk_text(&crosswalk);
    let crosswalk_json = render_crosswalk_json(&crosswalk);

    let snap_txt_path = format!("{out_dir}/admissibility_grammar.txt");
    let snap_json_path = format!("{out_dir}/admissibility_grammar.json");
    let cross_txt_path = format!("{out_dir}/passport_grammar_crosswalk.txt");
    let cross_json_path = format!("{out_dir}/passport_grammar_crosswalk.json");

    for (path, body) in [
        (&snap_txt_path, &snapshot_text),
        (&snap_json_path, &snapshot_json),
        (&cross_txt_path, &crosswalk_text),
        (&cross_json_path, &crosswalk_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_trial_transcript(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `trial-transcript`");
                return ExitCode::from(1);
            }
        }
    }
    let transcript = build_t11d_latency_ramp_fixture();
    let body = if json_mode {
        render_trial_transcript_json(&transcript)
    } else {
        render_trial_transcript_text(&transcript)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_trial_transcript_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `trial-transcript-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let transcript = build_t11d_latency_ramp_fixture();
    let text = render_trial_transcript_text(&transcript);
    let json = render_trial_transcript_json(&transcript);
    let txt_path = format!("{out_dir}/trial_transcript_v1.txt");
    let json_path = format!("{out_dir}/trial_transcript_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_execution_attestation(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `execution-attestation`");
                return ExitCode::from(1);
            }
        }
    }
    let receipt = build_t11e_live_attestation();
    let body = if json_mode {
        render_execution_attestation_json(&receipt)
    } else {
        render_execution_attestation_text(&receipt)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_execution_attestation_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `execution-attestation-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let receipt = build_t11e_live_attestation();
    let text = render_execution_attestation_text(&receipt);
    let json = render_execution_attestation_json(&receipt);
    let txt_path = format!("{out_dir}/execution_attestation_v1.txt");
    let json_path = format!("{out_dir}/execution_attestation_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_challenges(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `challenges`");
                return ExitCode::from(1);
            }
        }
    }
    let docket = collect_challenge_docket();
    let body = if json_mode {
        render_challenge_docket_json(&docket)
    } else {
        render_challenge_docket_text(&docket)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_challenges_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `challenges-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let docket = collect_challenge_docket();
    let text = render_challenge_docket_text(&docket);
    let json = render_challenge_docket_json(&docket);
    let txt_path = format!("{out_dir}/challenge_docket_v1.txt");
    let json_path = format!("{out_dir}/challenge_docket_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_contraindication(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        let arg = rest[idx].as_str();
        match arg {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `contraindication`");
                return ExitCode::from(1);
            }
        }
    }
    let snapshot = collect_contraindications();
    let body = if json_mode {
        render_contraindications_json(&snapshot)
    } else {
        render_contraindications_text(&snapshot)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_contraindications_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `contraindications-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let snapshot = collect_contraindications();
    let receipts_text = render_contraindications_text(&snapshot);
    let receipts_json = render_contraindications_json(&snapshot);
    let crosswalk_text = render_passport_crosswalk_text(&snapshot);
    let crosswalk_json = render_passport_crosswalk_json(&snapshot);
    let r_txt = format!("{out_dir}/contraindications_v1.txt");
    let r_json = format!("{out_dir}/contraindications_v1.json");
    let c_txt = format!("{out_dir}/passport_contraindication_crosswalk.txt");
    let c_json = format!("{out_dir}/passport_contraindication_crosswalk.json");
    for (path, body) in [
        (&r_txt, &receipts_text),
        (&r_json, &receipts_json),
        (&c_txt, &crosswalk_text),
        (&c_json, &crosswalk_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_coverage_holes(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `coverage-holes`");
                return ExitCode::from(1);
            }
        }
    }
    let snapshot = collect_coverage_holes();
    let body = if json_mode {
        render_coverage_hole_report_json(&snapshot)
    } else {
        render_coverage_hole_report_text(&snapshot)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_coverage_holes_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = rest[idx + 1].as_str();
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `coverage-holes-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let snapshot = collect_coverage_holes();
    let text = render_coverage_hole_report_text(&snapshot);
    let json = render_coverage_hole_report_json(&snapshot);
    let txt_path = format!("{out_dir}/coverage_holes_v1.txt");
    let json_path = format!("{out_dir}/coverage_holes_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn parse_registry_hash(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        let pair = hex.get(i * 2..i * 2 + 2)?;
        *byte = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(out)
}

fn run_activation_plan(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut registry_hash = KNOWN_S12_REGISTRY_HASH_V2;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            "--registry-hash" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--registry-hash` requires a hex argument");
                    return ExitCode::from(1);
                }
                if let Some(bytes) = parse_registry_hash(&rest[idx + 1]) {
                    registry_hash = bytes;
                } else {
                    eprintln!("dsfb-corpus: `--registry-hash` requires a 64-char hex string");
                    return ExitCode::from(1);
                }
                idx += 2;
            }
            other if other.starts_with("--registry-hash=") => {
                let hex = &other["--registry-hash=".len()..];
                if let Some(bytes) = parse_registry_hash(hex) {
                    registry_hash = bytes;
                } else {
                    eprintln!("dsfb-corpus: `--registry-hash=` requires a 64-char hex string");
                    return ExitCode::from(1);
                }
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-plan`");
                return ExitCode::from(1);
            }
        }
    }
    let plan = collect_activation_plan(registry_hash);
    let body = if json_mode {
        render_activation_plan_json(&plan)
    } else {
        render_activation_plan_text(&plan)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_activation_plan_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut registry_hash = KNOWN_S12_REGISTRY_HASH_V2;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            "--registry-hash" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--registry-hash` requires a hex argument");
                    return ExitCode::from(1);
                }
                if let Some(bytes) = parse_registry_hash(&rest[idx + 1]) {
                    registry_hash = bytes;
                } else {
                    eprintln!("dsfb-corpus: `--registry-hash` requires a 64-char hex string");
                    return ExitCode::from(1);
                }
                idx += 2;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-plan-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let plan = collect_activation_plan(registry_hash);
    let text = render_activation_plan_text(&plan);
    let json = render_activation_plan_json(&plan);
    let txt_path = format!("{out_dir}/activation_plan_v1.txt");
    let json_path = format!("{out_dir}/activation_plan_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_activation_plan_explain(rest: &[String]) -> ExitCode {
    let mut canonical_arg: Option<u32> = None;
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => match other.parse::<u32>() {
                Ok(n) if canonical_arg.is_none() => {
                    canonical_arg = Some(n);
                    idx += 1;
                }
                _ => {
                    eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-plan-explain`");
                    return ExitCode::from(1);
                }
            },
        }
    }
    let Some(id) = canonical_arg else {
        eprintln!("dsfb-corpus: `activation-plan-explain` requires a canonical_id argument");
        return ExitCode::from(1);
    };
    let Some(transcript) = build_transcript_for(DetectorCanonicalId(id)) else {
        eprintln!("dsfb-corpus: no canonical detector with id {id} in SEED");
        return ExitCode::from(2);
    };
    let body = if json_mode {
        render_transcript_json(&transcript)
    } else {
        render_transcript_text(&transcript)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_activation_plan_audit_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-plan-audit-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let audit = build_plan_audit();
    let text = render_audit_text(&audit);
    let json = render_audit_json(&audit);
    let txt_path = format!("{out_dir}/activation_plan_audit_v1.txt");
    let json_path = format!("{out_dir}/activation_plan_audit_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_activation_plan_diff(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut _old_path: Option<String> = None;
    let mut _new_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            "--old" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--old` requires a path argument");
                    return ExitCode::from(1);
                }
                _old_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            "--new" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--new` requires a path argument");
                    return ExitCode::from(1);
                }
                _new_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-plan-diff`");
                return ExitCode::from(1);
            }
        }
    }
    // S1.3b CLI implements the in-memory diff path against the
    // live plan twice (which by construction produces an empty
    // diff with zero counts — useful as a CI sanity check and as
    // a demo of the diff schema). External plan-file ingestion
    // is honestly deferred to S1.3c+ alongside TaskManifest
    // input parsing; the CLI surfaces the schema and prints the
    // empty-diff demonstration when --old/--new are supplied so
    // operators see the rendering before the file-parse path
    // exists.
    let live = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let diff = build_diff(&live, &live);
    let body = if json_mode {
        render_diff_json(&diff)
    } else {
        render_diff_text(&diff)
    };
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, &body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_task_manifest(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `task-manifest`");
                return ExitCode::from(1);
            }
        }
    }
    let m = seed_task_manifest();
    let body = if json_mode {
        render_task_manifest_json(&m)
    } else {
        render_task_manifest_text(&m)
    };
    emit_or_print(out_path, &body)
}

fn run_dataset_manifest(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `dataset-manifest`");
                return ExitCode::from(1);
            }
        }
    }
    let m = seed_dataset_manifest();
    let body = if json_mode {
        render_dataset_manifest_json(&m)
    } else {
        render_dataset_manifest_text(&m)
    };
    emit_or_print(out_path, &body)
}

fn run_activation_context(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-context`");
                return ExitCode::from(1);
            }
        }
    }
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let registry_hash = dsfb_gpu_atlas_corpus::activation::KNOWN_S12_REGISTRY_HASH_V2;
    let coverage_hash =
        dsfb_gpu_atlas_corpus::coverage_holes::compute_coverage_hole_hash_v1(&coverage);
    let contra_hash =
        dsfb_gpu_atlas_corpus::contraindication::compute_contraindication_hash_v1(&contras);
    let c = build_activation_context(&task, &dataset, registry_hash, coverage_hash, contra_hash);
    let body = if json_mode {
        render_activation_context_json(&c)
    } else {
        render_activation_context_text(&c)
    };
    emit_or_print(out_path, &body)
}

fn run_activation_context_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `activation-context-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let contras = collect_contraindications();
    let coverage = collect_coverage_holes();
    let registry_hash = dsfb_gpu_atlas_corpus::activation::KNOWN_S12_REGISTRY_HASH_V2;
    let coverage_hash =
        dsfb_gpu_atlas_corpus::coverage_holes::compute_coverage_hole_hash_v1(&coverage);
    let contra_hash =
        dsfb_gpu_atlas_corpus::contraindication::compute_contraindication_hash_v1(&contras);
    let c = build_activation_context(&task, &dataset, registry_hash, coverage_hash, contra_hash);
    let artifacts: [(String, String); 6] = [
        (
            format!("{out_dir}/task_manifest_v1.txt"),
            render_task_manifest_text(&task),
        ),
        (
            format!("{out_dir}/task_manifest_v1.json"),
            render_task_manifest_json(&task),
        ),
        (
            format!("{out_dir}/dataset_manifest_v1.txt"),
            render_dataset_manifest_text(&dataset),
        ),
        (
            format!("{out_dir}/dataset_manifest_v1.json"),
            render_dataset_manifest_json(&dataset),
        ),
        (
            format!("{out_dir}/activation_context_v1.txt"),
            render_activation_context_text(&c),
        ),
        (
            format!("{out_dir}/activation_context_v1.json"),
            render_activation_context_json(&c),
        ),
    ];
    for (path, body) in &artifacts {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn emit_or_print(out_path: Option<String>, body: &str) -> ExitCode {
    match out_path {
        None => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Some(path) => match std::fs::write(&path, body) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
                ExitCode::from(5)
            }
        },
    }
}

fn run_amendment_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `amendment-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_proof_of_life_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

fn run_amendment_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `amendment-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_proof_of_life_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/corpus_amendment_proposal_v1.txt");
    let json_path = format!("{out_dir}/corpus_amendment_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

fn run_t12_a_spc_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-a-spc-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_a_spc_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

fn run_t12_a_spc_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-a-spc-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_a_spc_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_a_spc_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_a_spc_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-b-scd-proposal` — render the T.12.b Sequential Change
/// Detection amendment proposal. Default text; `--json` switches
/// to JSON; `--out PATH` redirects to a file. Two invocations
/// produce byte-identical bytes because the proposal seed is
/// built deterministically and the renderers are deterministic.
fn run_t12_b_scd_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-b-scd-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_b_scd_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-b-scd-proposal-emit` — write the T.12.b SCD proposal's
/// text + JSON renderings to the bulk-artifact output directory
/// (default `crates/dsfb-gpu-atlas-corpus/out`). Used by the
/// 10-step ritual to pin the two artifacts the receipts
/// reference.
fn run_t12_b_scd_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-b-scd-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_b_scd_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_b_scd_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_b_scd_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-c-drift-proposal` — render the T.12.c Drift Detection
/// and Distribution-Distance Authority amendment proposal.
/// Default text; `--json` switches to JSON; `--out PATH`
/// redirects to a file. Two invocations produce byte-identical
/// bytes because the proposal seed is built deterministically
/// and the renderers are deterministic.
fn run_t12_c_drift_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-c-drift-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_c_drift_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-c-drift-proposal-emit` — write the T.12.c drift
/// proposal's text + JSON renderings to the bulk-artifact
/// output directory (default `crates/dsfb-gpu-atlas-corpus/out`).
/// Used by the 10-step ritual to pin the two artifacts the
/// receipts reference.
fn run_t12_c_drift_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-c-drift-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_c_drift_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_c_drift_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_c_drift_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-d-robust-proposal` — render the T.12.d Robust Statistics
/// amendment proposal. Default text; `--json` switches to JSON;
/// `--out PATH` redirects to a file. Two invocations produce
/// byte-identical bytes because the proposal seed is built
/// deterministically and the renderers are deterministic.
fn run_t12_d_robust_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-d-robust-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_d_robust_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-d-robust-proposal-emit` — write the T.12.d robust
/// proposal's text + JSON renderings to the bulk-artifact
/// output directory (default `crates/dsfb-gpu-atlas-corpus/out`).
fn run_t12_d_robust_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-d-robust-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_d_robust_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_d_robust_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_d_robust_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-e-spectral-proposal` — render the T.12.e Signal
/// Processing / Spectral / Wavelet amendment proposal. Default
/// text; `--json` switches to JSON; `--out PATH` redirects to
/// a file. Two invocations produce byte-identical bytes.
fn run_t12_e_spectral_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-e-spectral-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_e_spectral_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-e-spectral-proposal-emit` — write the T.12.e spectral
/// proposal's text + JSON renderings to the bulk-artifact
/// output directory.
fn run_t12_e_spectral_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-e-spectral-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_e_spectral_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_e_spectral_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_e_spectral_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-f-timeseries-proposal` — render the T.12.f Time-Series
/// Structure / Control Residuals amendment proposal. Default
/// text; `--json` switches to JSON; `--out PATH` redirects to
/// a file. Two invocations produce byte-identical bytes.
fn run_t12_f_timeseries_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-f-timeseries-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_f_timeseries_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-f-timeseries-proposal-emit` — write the T.12.f time-
/// series proposal's text + JSON renderings to the bulk-
/// artifact output directory.
fn run_t12_f_timeseries_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-f-timeseries-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_f_timeseries_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_f_timeseries_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_f_timeseries_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-g-graph-proposal` — render the T.12.g Graph / Topology
/// Anomaly amendment proposal. Default text; `--json` switches
/// to JSON; `--out PATH` redirects to a file.
fn run_t12_g_graph_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-g-graph-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_g_graph_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-g-graph-proposal-emit` — write the T.12.g graph
/// proposal's text + JSON renderings to the bulk-artifact
/// output directory.
fn run_t12_g_graph_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-g-graph-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_g_graph_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_g_graph_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_g_graph_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-h-dataquality-proposal` — render the T.12.h Data
/// Quality / Tabular / Database Integrity Constraints amendment
/// proposal.
fn run_t12_h_dataquality_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-h-dataquality-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_h_dataquality_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-h-dataquality-proposal-emit` — write the T.12.h data-
/// quality proposal's text + JSON renderings to the bulk-
/// artifact output directory.
fn run_t12_h_dataquality_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-h-dataquality-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_h_dataquality_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_h_dataquality_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_h_dataquality_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-i-observability-proposal` — render the T.12.i
/// Observability / Debugging amendment proposal.
fn run_t12_i_observability_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-i-observability-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_i_observability_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-i-observability-proposal-emit` — write the T.12.i
/// observability proposal's text + JSON renderings to the
/// bulk-artifact output directory.
fn run_t12_i_observability_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-i-observability-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_i_observability_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_i_observability_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_i_observability_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-j-biosignal-proposal` — render the T.12.j Medical /
/// Biosignal amendment proposal.
fn run_t12_j_biosignal_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-j-biosignal-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_j_biosignal_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-j-biosignal-proposal-emit` — write the T.12.j biosignal
/// proposal's text + JSON renderings to the bulk-artifact
/// output directory.
fn run_t12_j_biosignal_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-j-biosignal-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_j_biosignal_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_j_biosignal_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_j_biosignal_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-k-industrial-proposal` — render the T.12.k Industrial
/// / FDD / Condition Monitoring amendment proposal.
fn run_t12_k_industrial_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-k-industrial-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_k_industrial_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-k-industrial-proposal-emit` — write the T.12.k
/// industrial proposal's text + JSON renderings to the bulk-
/// artifact output directory.
fn run_t12_k_industrial_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-k-industrial-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_k_industrial_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_k_industrial_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_k_industrial_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-l-chemometrics-proposal` — render the T.12.l
/// Chemometrics amendment proposal.
fn run_t12_l_chemometrics_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-l-chemometrics-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_l_chemometrics_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-l-chemometrics-proposal-emit` — write the T.12.l
/// chemometrics proposal's text + JSON renderings to the
/// bulk-artifact output directory.
fn run_t12_l_chemometrics_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-l-chemometrics-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_l_chemometrics_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_l_chemometrics_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_l_chemometrics_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-m-rf-proposal` — render the T.12.m RF /
/// Communications amendment proposal.
fn run_t12_m_rf_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-m-rf-proposal`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_m_rf_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-m-rf-proposal-emit` — write the T.12.m RF /
/// Communications proposal's text + JSON renderings to the
/// bulk-artifact output directory.
fn run_t12_m_rf_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-m-rf-proposal-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_m_rf_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_m_rf_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_m_rf_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-n-econometrics-reliability-proposal` — render the
/// T.12.n Econometrics + Reliability / Survival amendment
/// proposal.
fn run_t12_n_econometrics_reliability_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-n-econometrics-reliability-proposal`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_n_econometrics_reliability_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-n-econometrics-reliability-proposal-emit` — write the
/// T.12.n Econometrics + Reliability / Survival proposal's
/// text + JSON renderings to the bulk-artifact output
/// directory.
fn run_t12_n_econometrics_reliability_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-n-econometrics-reliability-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_n_econometrics_reliability_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_n_econometrics_reliability_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_n_econometrics_reliability_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-o-streaming-sketches-proposal` — render the T.12.o
/// Streaming Sketches amendment proposal.
fn run_t12_o_streaming_sketches_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-o-streaming-sketches-proposal`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_o_streaming_sketches_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-o-streaming-sketches-proposal-emit` — write the T.12.o
/// Streaming Sketches proposal's text + JSON renderings to the
/// bulk-artifact output directory.
fn run_t12_o_streaming_sketches_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-o-streaming-sketches-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_o_streaming_sketches_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_o_streaming_sketches_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_o_streaming_sketches_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-p-information-theory-proposal` — render the T.12.p
/// Information Theory catch-up amendment proposal.
fn run_t12_p_information_theory_proposal(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-p-information-theory-proposal`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_t12_p_information_theory_proposal();
    let body = if json_mode {
        render_amendment_proposal_json(&p)
    } else {
        render_amendment_proposal_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `t12-p-information-theory-proposal-emit` — write the T.12.p
/// Information Theory catch-up proposal's text + JSON renderings
/// to the bulk-artifact output directory.
fn run_t12_p_information_theory_proposal_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-p-information-theory-proposal-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = seed_t12_p_information_theory_proposal();
    let text = render_amendment_proposal_text(&p);
    let json = render_amendment_proposal_json(&p);
    let txt_path = format!("{out_dir}/t12_p_information_theory_proposal_v1.txt");
    let json_path = format!("{out_dir}/t12_p_information_theory_proposal_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// Helper: render either text or JSON for a consolidation
/// report, with optional `--out PATH` overlay. Used by the
/// three single-render T.12.consolidate subcommands.
fn render_consolidation_artifact(
    rest: &[String],
    subcommand: &str,
    text_renderer: impl Fn(&dsfb_gpu_atlas_corpus::consolidate::ConsolidationReport) -> String,
    json_renderer: impl Fn(&dsfb_gpu_atlas_corpus::consolidate::ConsolidationReport) -> String,
) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `{subcommand}`");
                return ExitCode::from(1);
            }
        }
    }
    let r = build_consolidation_report();
    let body = if json_mode {
        json_renderer(&r)
    } else {
        text_renderer(&r)
    };
    emit_or_print(out_path, &body)
}

/// `t12-consolidate-report` — render the T.12.consolidate
/// top-level consolidation report.
fn run_t12_consolidate_report(rest: &[String]) -> ExitCode {
    render_consolidation_artifact(
        rest,
        "t12-consolidate-report",
        render_consolidation_report_text,
        render_consolidation_report_json,
    )
}

/// `t12-corpus-v2-freeze` — render the corpus_v2 freeze
/// receipt (compact summary suitable for the bulk-artifact
/// emit).
fn run_t12_corpus_v2_freeze(rest: &[String]) -> ExitCode {
    render_consolidation_artifact(
        rest,
        "t12-corpus-v2-freeze",
        render_corpus_v2_freeze_text,
        render_corpus_v2_freeze_json,
    )
}

/// `t12-expansion-index` — render the T.12 expansion index
/// (one row per admitted CanonicalAddition record across all
/// proposals, sorted by canonical_id).
fn run_t12_expansion_index(rest: &[String]) -> ExitCode {
    render_consolidation_artifact(
        rest,
        "t12-expansion-index",
        render_t12_expansion_index_text,
        render_t12_expansion_index_json,
    )
}

/// Helper: emit all three text + JSON artifacts to an output
/// directory. Used by the three T.12.consolidate `-emit`
/// subcommands.
fn emit_consolidation_artifacts(
    rest: &[String],
    subcommand: &str,
    txt_filename: &str,
    json_filename: &str,
    text_renderer: impl Fn(&dsfb_gpu_atlas_corpus::consolidate::ConsolidationReport) -> String,
    json_renderer: impl Fn(&dsfb_gpu_atlas_corpus::consolidate::ConsolidationReport) -> String,
) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `{subcommand}`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let r = build_consolidation_report();
    let text = text_renderer(&r);
    let json = json_renderer(&r);
    let txt_path = format!("{out_dir}/{txt_filename}");
    let json_path = format!("{out_dir}/{json_filename}");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-consolidate-report-emit` — write the full
/// consolidation-report text + JSON renderings to the bulk-
/// artifact output directory.
fn run_t12_consolidate_report_emit(rest: &[String]) -> ExitCode {
    emit_consolidation_artifacts(
        rest,
        "t12-consolidate-report-emit",
        "t12_consolidation_report_v1.txt",
        "t12_consolidation_report_v1.json",
        render_consolidation_report_text,
        render_consolidation_report_json,
    )
}

/// `t12-corpus-v2-freeze-emit` — write the corpus_v2 freeze
/// receipt text + JSON to the bulk-artifact output directory.
fn run_t12_corpus_v2_freeze_emit(rest: &[String]) -> ExitCode {
    emit_consolidation_artifacts(
        rest,
        "t12-corpus-v2-freeze-emit",
        "corpus_v2_freeze_v1.txt",
        "corpus_v2_freeze_v1.json",
        render_corpus_v2_freeze_text,
        render_corpus_v2_freeze_json,
    )
}

/// `t12-expansion-index-emit` — write the T.12 expansion
/// index text + JSON to the bulk-artifact output directory.
fn run_t12_expansion_index_emit(rest: &[String]) -> ExitCode {
    emit_consolidation_artifacts(
        rest,
        "t12-expansion-index-emit",
        "t12_expansion_index_v1.txt",
        "t12_expansion_index_v1.json",
        render_t12_expansion_index_text,
        render_t12_expansion_index_json,
    )
}

/// `ff1-passport-index` — render the FF.1 T.12 ratified
/// passport index (one materialised passport per ratified
/// CanonicalAddition record).
fn run_ff1_passport_index(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff1-passport-index`");
                return ExitCode::from(1);
            }
        }
    }
    let i = build_ff1_passport_index();
    let body = if json_mode {
        render_ff1_passport_index_json(&i)
    } else {
        render_ff1_passport_index_text(&i)
    };
    emit_or_print(out_path, &body)
}

/// `ff1-passport-index-emit` — write the FF.1 passport index
/// text + JSON renderings to the bulk-artifact output
/// directory.
fn run_ff1_passport_index_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff1-passport-index-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let i = build_ff1_passport_index();
    let text = render_ff1_passport_index_text(&i);
    let json = render_ff1_passport_index_json(&i);
    let txt_path = format!("{out_dir}/ff1_passport_index_v1.txt");
    let json_path = format!("{out_dir}/ff1_passport_index_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff1-materialisation-report` — render the FF.1
/// materialisation report (compact summary suitable for the
/// bulk-artifact emit).
fn run_ff1_materialisation_report(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff1-materialisation-report`");
                return ExitCode::from(1);
            }
        }
    }
    let r = build_ff1_materialisation_report();
    let body = if json_mode {
        render_ff1_materialisation_report_json(&r)
    } else {
        render_ff1_materialisation_report_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `ff1-materialisation-report-emit` — write the FF.1
/// materialisation report text + JSON to the bulk-artifact
/// output directory.
fn run_ff1_materialisation_report_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `ff1-materialisation-report-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let r = build_ff1_materialisation_report();
    let text = render_ff1_materialisation_report_text(&r);
    let json = render_ff1_materialisation_report_json(&r);
    let txt_path = format!("{out_dir}/ff1_materialisation_report_v1.txt");
    let json_path = format!("{out_dir}/ff1_materialisation_report_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff2-gate` — render the FF.2 activation ratification gate
/// over the live (SEED + FF.1 passport index) candidate set.
fn run_ff2_gate(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff2-gate`");
                return ExitCode::from(1);
            }
        }
    }
    let g = build_ff2_activation_ratification_gate();
    let body = if json_mode {
        render_ff2_gate_json(&g)
    } else {
        render_ff2_gate_text(&g)
    };
    emit_or_print(out_path, &body)
}

/// `ff2-gate-emit` — write the FF.2 gate text + JSON to the
/// bulk-artifact output directory.
fn run_ff2_gate_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff2-gate-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let g = build_ff2_activation_ratification_gate();
    let text = render_ff2_gate_text(&g);
    let json = render_ff2_gate_json(&g);
    let txt_path = format!("{out_dir}/ff2_activation_ratification_gate_v1.txt");
    let json_path = format!("{out_dir}/ff2_activation_ratification_gate_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff2-gate-summary` — render the FF.2 gate summary (gate
/// plus panel-locked non-claim block).
fn run_ff2_gate_summary(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff2-gate-summary`");
                return ExitCode::from(1);
            }
        }
    }
    let s = build_ff2_activation_ratification_gate_summary();
    let body = if json_mode {
        render_ff2_gate_summary_json(&s)
    } else {
        render_ff2_gate_summary_text(&s)
    };
    emit_or_print(out_path, &body)
}

/// `ff2-gate-summary-emit` — write the FF.2 gate summary text
/// + JSON to the bulk-artifact output directory.
fn run_ff2_gate_summary_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff2-gate-summary-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let s = build_ff2_activation_ratification_gate_summary();
    let text = render_ff2_gate_summary_text(&s);
    let json = render_ff2_gate_summary_json(&s);
    let txt_path = format!("{out_dir}/ff2_activation_ratification_gate_summary_v1.txt");
    let json_path = format!("{out_dir}/ff2_activation_ratification_gate_summary_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff3-gate` — render the FF.3 registry-generation gate over
/// the live (SEED + FF.1 passport index) candidate set.
fn run_ff3_gate(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff3-gate`");
                return ExitCode::from(1);
            }
        }
    }
    let g = build_ff3_registry_generation_gate();
    let body = if json_mode {
        render_ff3_gate_json(&g)
    } else {
        render_ff3_gate_text(&g)
    };
    emit_or_print(out_path, &body)
}

/// `ff3-gate-emit` — write the FF.3 gate text + JSON to the
/// bulk-artifact output directory.
fn run_ff3_gate_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff3-gate-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let g = build_ff3_registry_generation_gate();
    let text = render_ff3_gate_text(&g);
    let json = render_ff3_gate_json(&g);
    let txt_path = format!("{out_dir}/ff3_registry_generation_gate_v1.txt");
    let json_path = format!("{out_dir}/ff3_registry_generation_gate_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff3-gate-summary` — render the FF.3 gate summary (gate plus
/// panel-locked non-claim block).
fn run_ff3_gate_summary(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff3-gate-summary`");
                return ExitCode::from(1);
            }
        }
    }
    let s = build_ff3_registry_generation_gate_summary();
    let body = if json_mode {
        render_ff3_gate_summary_json(&s)
    } else {
        render_ff3_gate_summary_text(&s)
    };
    emit_or_print(out_path, &body)
}

/// `ff3-gate-summary-emit` — write the FF.3 gate summary text +
/// JSON to the bulk-artifact output directory.
fn run_ff3_gate_summary_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff3-gate-summary-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let s = build_ff3_registry_generation_gate_summary();
    let text = render_ff3_gate_summary_text(&s);
    let json = render_ff3_gate_summary_json(&s);
    let txt_path = format!("{out_dir}/ff3_registry_generation_gate_summary_v1.txt");
    let json_path = format!("{out_dir}/ff3_registry_generation_gate_summary_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff4-policy` — render the FF.4 README authority-boundary
/// policy artifact (pinned canonical block + required +
/// forbidden substring sets + upstream anchor hashes).
fn run_ff4_policy(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff4-policy`");
                return ExitCode::from(1);
            }
        }
    }
    let p = build_ff4_readme_authority_boundary_policy();
    let body = if json_mode {
        render_ff4_policy_json(&p)
    } else {
        render_ff4_policy_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `ff4-policy-emit` — write the FF.4 policy text + JSON to
/// the bulk-artifact output directory.
fn run_ff4_policy_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff4-policy-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = build_ff4_readme_authority_boundary_policy();
    let text = render_ff4_policy_text(&p);
    let json = render_ff4_policy_json(&p);
    let txt_path = format!("{out_dir}/ff4_readme_authority_boundary_policy_v1.txt");
    let json_path = format!("{out_dir}/ff4_readme_authority_boundary_policy_v1.json");
    for (path, body) in [(&txt_path, &text), (&json_path, &json)] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff4-authority-boundary-block` — emit the canonical
/// authority-boundary block text (the verbatim block operators
/// should copy into the README front-door area).
fn run_ff4_authority_boundary_block(rest: &[String]) -> ExitCode {
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff4-authority-boundary-block`");
                return ExitCode::from(1);
            }
        }
    }
    let body = render_ff4_authority_boundary_block();
    emit_or_print(out_path, &body)
}

/// `ff5-policy` — render the FF.5 proposal-schema upgrade
/// policy artifact (doctrine + migration table + pinned
/// upstream anchor hashes).
fn run_ff5_policy(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff5-policy`");
                return ExitCode::from(1);
            }
        }
    }
    let p = build_proposal_schema_upgrade_policy();
    let body = if json_mode {
        render_ff5_policy_json(&p)
    } else {
        render_ff5_policy_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `ff5-policy-emit` — write the FF.5 policy + migration table
/// text + JSON to the bulk-artifact output directory.
fn run_ff5_policy_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff5-policy-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let p = build_proposal_schema_upgrade_policy();
    let policy_text = render_ff5_policy_text(&p);
    let policy_json = render_ff5_policy_json(&p);
    let table_text = dsfb_gpu_atlas_corpus::proposal_schema_policy::render_ff5_migration_table_text(
        &p.migration_table,
    );
    let table_json = dsfb_gpu_atlas_corpus::proposal_schema_policy::render_ff5_migration_table_json(
        &p.migration_table,
    );
    let policy_txt_path = format!("{out_dir}/proposal_schema_upgrade_policy_v1.txt");
    let policy_json_path = format!("{out_dir}/proposal_schema_upgrade_policy_v1.json");
    let table_txt_path = format!("{out_dir}/proposal_schema_migration_table_v1.txt");
    let table_json_path = format!("{out_dir}/proposal_schema_migration_table_v1.json");
    for (path, body) in [
        (&policy_txt_path, &policy_text),
        (&policy_json_path, &policy_json),
        (&table_txt_path, &table_text),
        (&table_json_path, &table_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `ff5-migration-table` — render the FF.5 migration table
/// (empty at FF.5 baseline state; future schema-upgrade commits
/// append rows).
fn run_ff5_migration_table(rest: &[String]) -> ExitCode {
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `ff5-migration-table`");
                return ExitCode::from(1);
            }
        }
    }
    let p = build_proposal_schema_upgrade_policy();
    let body = render_ff5_migration_table_text(&p.migration_table);
    emit_or_print(out_path, &body)
}

/// `s1-3d-plan` — render the S1.3d budget pruning plan
/// (per-candidate decision list + tie-break transcript +
/// per-reason disable counts + pinned upstream anchors).
///
/// The plan is built from the live FF.3-eligible candidate set
/// under the panel-locked default task budget and the empty
/// redundancy cluster set. Two invocations produce byte-identical
/// output — the determinism gate the panel-required negatives
/// pin in the acceptance suite.
fn run_s1_3d_plan(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3d-plan`");
                return ExitCode::from(1);
            }
        }
    }
    let summary = build_budgeted_activation_summary();
    let body = if json_mode {
        render_s13d_plan_json(&summary.plan)
    } else {
        render_s13d_plan_text(&summary.plan)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3d-plan-emit` — write the S1.3d plan + redundancy report
/// + summary as text + JSON to the bulk-artifact output directory.
///
/// Six files total: `s1_3d_budget_pruning_plan_v1.{txt,json}`,
/// `s1_3d_redundancy_suppression_v1.{txt,json}`,
/// `s1_3d_budgeted_activation_summary_v1.{txt,json}`. All six
/// are byte-stable across two emits; the determinism gate is
/// enforced by the acceptance suite.
fn run_s1_3d_plan_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3d-plan-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let summary = build_budgeted_activation_summary();
    let plan_text = render_s13d_plan_text(&summary.plan);
    let plan_json = render_s13d_plan_json(&summary.plan);
    let redundancy_text = render_s13d_redundancy_text(&summary.redundancy_report);
    let redundancy_json = render_s13d_redundancy_json(&summary.redundancy_report);
    let summary_text = render_s13d_summary_text(&summary);
    let summary_json = render_s13d_summary_json(&summary);
    let plan_txt_path = format!("{out_dir}/s1_3d_budget_pruning_plan_v1.txt");
    let plan_json_path = format!("{out_dir}/s1_3d_budget_pruning_plan_v1.json");
    let red_txt_path = format!("{out_dir}/s1_3d_redundancy_suppression_v1.txt");
    let red_json_path = format!("{out_dir}/s1_3d_redundancy_suppression_v1.json");
    let sum_txt_path = format!("{out_dir}/s1_3d_budgeted_activation_summary_v1.txt");
    let sum_json_path = format!("{out_dir}/s1_3d_budgeted_activation_summary_v1.json");
    for (path, body) in [
        (&plan_txt_path, &plan_text),
        (&plan_json_path, &plan_json),
        (&red_txt_path, &redundancy_text),
        (&red_json_path, &redundancy_json),
        (&sum_txt_path, &summary_text),
        (&sum_json_path, &summary_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s1-3d-redundancy` — render the S1.3d redundancy suppression
/// report (declared clusters + retained representatives +
/// suppression count). Empty at S1.3d baseline because no
/// production clusters have been declared yet; non-empty when
/// callers inject pressure-bearing clusters through the test
/// harness.
fn run_s1_3d_redundancy(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3d-redundancy`");
                return ExitCode::from(1);
            }
        }
    }
    let summary = build_budgeted_activation_summary();
    let body = if json_mode {
        render_s13d_redundancy_json(&summary.redundancy_report)
    } else {
        render_s13d_redundancy_text(&summary.redundancy_report)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3d-summary` — render the top-level S1.3d budgeted
/// activation summary META-hash wrapping the plan + redundancy
/// report. Useful for operators who want one hash that pins the
/// entire S1.3d state without consuming the per-decision
/// transcript.
fn run_s1_3d_summary(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3d-summary`");
                return ExitCode::from(1);
            }
        }
    }
    let summary = build_budgeted_activation_summary();
    let body = if json_mode {
        render_s13d_summary_json(&summary)
    } else {
        render_s13d_summary_text(&summary)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3e-plan` — render the S1.3e kernel plan (per-family
/// schedule + parameter table + nine pinned upstream anchors +
/// kernel_plan_hash_v1).
///
/// The plan is built deterministically from the live S1.3d
/// summary. Two invocations produce byte-identical output —
/// the determinism gate the acceptance suite pins.
fn run_s1_3e_plan(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3e-plan`");
                return ExitCode::from(1);
            }
        }
    }
    let plan = build_kernel_plan_v1();
    let body = if json_mode {
        render_kernel_plan_json(&plan)
    } else {
        render_kernel_plan_text(&plan)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3e-plan-emit` — write the S1.3e plan + family schedule +
/// parameter table as text + JSON to the bulk-artifact output
/// directory. Six files total; all six are byte-stable across
/// two emits.
fn run_s1_3e_plan_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3e-plan-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let plan = build_kernel_plan_v1();
    let schedule = build_kernel_family_schedule_v1();
    let table = build_kernel_parameter_table_v1();
    let plan_text = render_kernel_plan_text(&plan);
    let plan_json = render_kernel_plan_json(&plan);
    let sched_text = render_kernel_family_schedule_text(&schedule);
    let sched_json = render_kernel_family_schedule_json(&schedule);
    let table_text = render_kernel_parameter_table_text(&table);
    let table_json = render_kernel_parameter_table_json(&table);
    let plan_txt_path = format!("{out_dir}/s1_3e_kernel_plan_v1.txt");
    let plan_json_path = format!("{out_dir}/s1_3e_kernel_plan_v1.json");
    let sched_txt_path = format!("{out_dir}/s1_3e_kernel_family_schedule_v1.txt");
    let sched_json_path = format!("{out_dir}/s1_3e_kernel_family_schedule_v1.json");
    let table_txt_path = format!("{out_dir}/s1_3e_kernel_parameter_table_v1.txt");
    let table_json_path = format!("{out_dir}/s1_3e_kernel_parameter_table_v1.json");
    for (path, body) in [
        (&plan_txt_path, &plan_text),
        (&plan_json_path, &plan_json),
        (&sched_txt_path, &sched_text),
        (&sched_json_path, &sched_json),
        (&table_txt_path, &table_text),
        (&table_json_path, &table_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s1-3e-schedule` — render the S1.3e family schedule.
fn run_s1_3e_schedule(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3e-schedule`");
                return ExitCode::from(1);
            }
        }
    }
    let schedule = build_kernel_family_schedule_v1();
    let body = if json_mode {
        render_kernel_family_schedule_json(&schedule)
    } else {
        render_kernel_family_schedule_text(&schedule)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3e-parameter-table` — render the S1.3e parameter table.
fn run_s1_3e_parameter_table(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3e-parameter-table`");
                return ExitCode::from(1);
            }
        }
    }
    let table = build_kernel_parameter_table_v1();
    let body = if json_mode {
        render_kernel_parameter_table_json(&table)
    } else {
        render_kernel_parameter_table_text(&table)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3f-authority-chain` --- render the S1.3f authority
/// chain (activation binding + kernel-plan binding +
/// linkage anchors + corpus anchors +
/// casefile_v2_authority_chain_hash_v1).
fn run_s1_3f_authority_chain(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3f-authority-chain`");
                return ExitCode::from(1);
            }
        }
    }
    let chain = build_casefile_v2_authority_chain();
    let body = if json_mode {
        render_authority_chain_json(&chain)
    } else {
        render_authority_chain_text(&chain)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3f-authority-chain-emit` --- write the activation
/// binding + kernel-plan binding + authority chain as text +
/// JSON to the bulk-artifact output directory. Six files
/// total; all six are byte-stable across two emits.
fn run_s1_3f_authority_chain_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3f-authority-chain-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let chain = build_casefile_v2_authority_chain();
    let chain_text = render_authority_chain_text(&chain);
    let chain_json = render_authority_chain_json(&chain);
    let act_text = render_activation_binding_text(&chain.activation_binding);
    let act_json = render_activation_binding_json(&chain.activation_binding);
    let kp_text = render_kernel_plan_binding_text(&chain.kernel_plan_binding);
    let kp_json = render_kernel_plan_binding_json(&chain.kernel_plan_binding);
    let chain_txt_path = format!("{out_dir}/casefile_v2_authority_chain_v1.txt");
    let chain_json_path = format!("{out_dir}/casefile_v2_authority_chain_v1.json");
    let act_txt_path = format!("{out_dir}/casefile_v2_activation_binding_v1.txt");
    let act_json_path = format!("{out_dir}/casefile_v2_activation_binding_v1.json");
    let kp_txt_path = format!("{out_dir}/casefile_v2_kernel_plan_binding_v1.txt");
    let kp_json_path = format!("{out_dir}/casefile_v2_kernel_plan_binding_v1.json");
    for (path, body) in [
        (&chain_txt_path, &chain_text),
        (&chain_json_path, &chain_json),
        (&act_txt_path, &act_text),
        (&act_json_path, &act_json),
        (&kp_txt_path, &kp_text),
        (&kp_json_path, &kp_json),
    ] {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s1-3f-activation-binding` --- render the activation
/// binding section only.
fn run_s1_3f_activation_binding(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3f-activation-binding`");
                return ExitCode::from(1);
            }
        }
    }
    let chain = build_casefile_v2_authority_chain();
    let body = if json_mode {
        render_activation_binding_json(&chain.activation_binding)
    } else {
        render_activation_binding_text(&chain.activation_binding)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3f-kernel-plan-binding` --- render the kernel-plan
/// binding section only.
fn run_s1_3f_kernel_plan_binding(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3f-kernel-plan-binding`");
                return ExitCode::from(1);
            }
        }
    }
    let chain = build_casefile_v2_authority_chain();
    let body = if json_mode {
        render_kernel_plan_binding_json(&chain.kernel_plan_binding)
    } else {
        render_kernel_plan_binding_text(&chain.kernel_plan_binding)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3g-binding` --- render the S1.3g top-level OTel
/// binding receipt (span + metric + log + resource bindings
/// + corpus authority anchors + receipt hash).
fn run_s1_3g_binding(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3g-binding`");
                return ExitCode::from(1);
            }
        }
    }
    let r = build_otel_binding_receipt();
    let body = if json_mode {
        render_otel_binding_receipt_json(&r)
    } else {
        render_otel_binding_receipt_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s1-3g-binding-emit` --- write the receipt + 4 per-binding
/// sections as text + JSON to the bulk-artifact output
/// directory. Ten files total; all ten are byte-stable across
/// two emits.
fn run_s1_3g_binding_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s1-3g-binding-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let r = build_otel_binding_receipt();
    let receipt_text = render_otel_binding_receipt_text(&r);
    let receipt_json = render_otel_binding_receipt_json(&r);
    let span_text = render_span_binding_text(&r.span_binding);
    let span_json = render_span_binding_json(&r.span_binding);
    let metric_text = render_metric_binding_text(&r.metric_binding);
    let metric_json = render_metric_binding_json(&r.metric_binding);
    let log_text = render_log_binding_text(&r.log_binding);
    let log_json = render_log_binding_json(&r.log_binding);
    let res_text = render_resource_binding_text(&r.resource_binding);
    let res_json = render_resource_binding_json(&r.resource_binding);
    let paths_and_bodies: [(String, String); 10] = [
        (
            format!("{out_dir}/otel_binding_receipt_v1.txt"),
            receipt_text,
        ),
        (
            format!("{out_dir}/otel_binding_receipt_v1.json"),
            receipt_json,
        ),
        (format!("{out_dir}/otel_span_binding_v1.txt"), span_text),
        (format!("{out_dir}/otel_span_binding_v1.json"), span_json),
        (format!("{out_dir}/otel_metric_binding_v1.txt"), metric_text),
        (
            format!("{out_dir}/otel_metric_binding_v1.json"),
            metric_json,
        ),
        (format!("{out_dir}/otel_log_binding_v1.txt"), log_text),
        (format!("{out_dir}/otel_log_binding_v1.json"), log_json),
        (format!("{out_dir}/otel_resource_binding_v1.txt"), res_text),
        (format!("{out_dir}/otel_resource_binding_v1.json"), res_json),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s1-3g-span-binding` --- render the span binding section only.
fn run_s1_3g_span_binding(rest: &[String]) -> ExitCode {
    run_one_binding(rest, "s1-3g-span-binding", |b| {
        (
            render_span_binding_text(&b.span_binding),
            render_span_binding_json(&b.span_binding),
        )
    })
}

/// `s1-3g-metric-binding` --- render the metric binding section only.
fn run_s1_3g_metric_binding(rest: &[String]) -> ExitCode {
    run_one_binding(rest, "s1-3g-metric-binding", |b| {
        (
            render_metric_binding_text(&b.metric_binding),
            render_metric_binding_json(&b.metric_binding),
        )
    })
}

/// `s1-3g-log-binding` --- render the log binding section only.
fn run_s1_3g_log_binding(rest: &[String]) -> ExitCode {
    run_one_binding(rest, "s1-3g-log-binding", |b| {
        (
            render_log_binding_text(&b.log_binding),
            render_log_binding_json(&b.log_binding),
        )
    })
}

/// `s1-3g-resource-binding` --- render the resource binding section only.
fn run_s1_3g_resource_binding(rest: &[String]) -> ExitCode {
    run_one_binding(rest, "s1-3g-resource-binding", |b| {
        (
            render_resource_binding_text(&b.resource_binding),
            render_resource_binding_json(&b.resource_binding),
        )
    })
}

/// Shared flag parser for the 4 per-binding S1.3g render
/// subcommands. The closure picks which binding to render
/// from the live receipt.
fn run_one_binding(
    rest: &[String],
    subcmd_name: &str,
    pick: impl FnOnce(
        &dsfb_gpu_atlas_corpus::s1_3g_otel_binding::OTelBindingReceiptTypesV1,
    ) -> (String, String),
) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `{subcmd_name}`");
                return ExitCode::from(1);
            }
        }
    }
    let r = build_otel_binding_receipt();
    let (text, json) = pick(&r);
    let body = if json_mode { json } else { text };
    emit_or_print(out_path, &body)
}

/// `t12-prov-report` --- render the T.12.PROV provenance
/// credit report (scientist credit index + source
/// bibliography index + corpus authority anchors + the
/// panel-locked DSFB-credit-note).
fn run_t12_prov_report(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-prov-report`");
                return ExitCode::from(1);
            }
        }
    }
    let r = build_provenance_credit_report();
    let body = if json_mode {
        render_provenance_credit_report_json(&r)
    } else {
        render_provenance_credit_report_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `t12-prov-report-emit` --- write the report + 2 sub-indexes
/// as text + JSON to the bulk-artifact output directory. Six
/// files total; all six are byte-stable across two emits.
fn run_t12_prov_report_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `t12-prov-report-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let report = build_provenance_credit_report();
    let credit_idx = build_scientist_credit_index();
    let biblio_idx = build_source_bibliography_index();
    let paths_and_bodies: [(String, String); 6] = [
        (
            format!("{out_dir}/t12_provenance_credit_report_v1.txt"),
            render_provenance_credit_report_text(&report),
        ),
        (
            format!("{out_dir}/t12_provenance_credit_report_v1.json"),
            render_provenance_credit_report_json(&report),
        ),
        (
            format!("{out_dir}/scientist_credit_index_v1.txt"),
            render_scientist_credit_index_text(&credit_idx),
        ),
        (
            format!("{out_dir}/scientist_credit_index_v1.json"),
            render_scientist_credit_index_json(&credit_idx),
        ),
        (
            format!("{out_dir}/source_bibliography_index_v1.txt"),
            render_source_bibliography_index_text(&biblio_idx),
        ),
        (
            format!("{out_dir}/source_bibliography_index_v1.json"),
            render_source_bibliography_index_json(&biblio_idx),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `t12-prov-scientist-credit-index` --- render only the
/// scientist credit index section.
fn run_t12_prov_scientist_credit_index(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-prov-scientist-credit-index`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let i = build_scientist_credit_index();
    let body = if json_mode {
        render_scientist_credit_index_json(&i)
    } else {
        render_scientist_credit_index_text(&i)
    };
    emit_or_print(out_path, &body)
}

/// `t12-prov-source-bibliography-index` --- render only the
/// source bibliography index section.
fn run_t12_prov_source_bibliography_index(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `t12-prov-source-bibliography-index`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let i = build_source_bibliography_index();
    let body = if json_mode {
        render_source_bibliography_index_json(&i)
    } else {
        render_source_bibliography_index_text(&i)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-1-receipt` --- render the S-PERF.1 baseline
/// uninstrumented device-traffic receipt (text by default,
/// JSON via `--json`). The baseline declares device identity
/// for the RTX 4080 SUPER reference host with every
/// measurement field zero; later S-PERF.* commits will
/// replace these zeros with measured values.
fn run_s_perf_1_receipt(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-1-receipt`");
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_baseline_uninstrumented_receipt();
    let body = if json_mode {
        render_device_traffic_receipt_json(&r)
    } else {
        render_device_traffic_receipt_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-1-receipt-emit` --- write the baseline receipt
/// (text + JSON) and the panel-locked bandwidth-claim policy
/// (text + JSON) to the bulk-artifact output directory. Four
/// files total; all four are byte-stable across two emits.
fn run_s_perf_1_receipt_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-1-receipt-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let receipt = seed_baseline_uninstrumented_receipt();
    let policy = build_panel_locked_bandwidth_claim_policy();
    let paths_and_bodies: [(String, String); 4] = [
        (
            format!("{out_dir}/s_perf_1_device_traffic_receipt_v1.txt"),
            render_device_traffic_receipt_text(&receipt),
        ),
        (
            format!("{out_dir}/s_perf_1_device_traffic_receipt_v1.json"),
            render_device_traffic_receipt_json(&receipt),
        ),
        (
            format!("{out_dir}/s_perf_1_bandwidth_claim_policy_v1.txt"),
            render_bandwidth_claim_policy_text(&policy),
        ),
        (
            format!("{out_dir}/s_perf_1_bandwidth_claim_policy_v1.json"),
            render_bandwidth_claim_policy_json(&policy),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-1-policy` --- render the panel-locked bandwidth-
/// claim policy (text by default, JSON via `--json`).
fn run_s_perf_1_policy(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-1-policy`");
                return ExitCode::from(1);
            }
        }
    }
    let p = build_panel_locked_bandwidth_claim_policy();
    let body = if json_mode {
        render_bandwidth_claim_policy_json(&p)
    } else {
        render_bandwidth_claim_policy_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-1-policy-emit` --- write the policy (text + JSON)
/// to the bulk-artifact output directory.
fn run_s_perf_1_policy_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-1-policy-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let policy = build_panel_locked_bandwidth_claim_policy();
    let paths_and_bodies: [(String, String); 2] = [
        (
            format!("{out_dir}/s_perf_1_bandwidth_claim_policy_v1.txt"),
            render_bandwidth_claim_policy_text(&policy),
        ),
        (
            format!("{out_dir}/s_perf_1_bandwidth_claim_policy_v1.json"),
            render_bandwidth_claim_policy_json(&policy),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-2-pipeline` --- render the S-PERF.2 baseline
/// Layer-A pipeline schema (text by default, JSON via
/// `--json`).
fn run_s_perf_2_pipeline(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-2-pipeline`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_baseline_layer_a_pipeline();
    let body = if json_mode {
        render_layer_a_resident_pipeline_json(&p)
    } else {
        render_layer_a_resident_pipeline_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-2-residency-receipt` --- render the S-PERF.2
/// baseline Layer-A residency receipt.
fn run_s_perf_2_residency_receipt(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-2-residency-receipt`");
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_baseline_layer_a_residency_receipt();
    let body = if json_mode {
        render_layer_a_device_residency_receipt_json(&r)
    } else {
        render_layer_a_device_residency_receipt_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-2-traffic-receipt` --- render the S-PERF.2
/// baseline Layer-A traffic receipt (the top-level META-hash
/// envelope binding pipeline + residency receipt + S-PERF.1
/// reference + court-authority anchors).
fn run_s_perf_2_traffic_receipt(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-2-traffic-receipt`");
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_baseline_layer_a_traffic_receipt();
    let body = if json_mode {
        render_layer_a_traffic_receipt_json(&r)
    } else {
        render_layer_a_traffic_receipt_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-2-receipts-emit` --- write the baseline pipeline,
/// the residency receipt, and the traffic receipt (each as
/// text + JSON) to the bulk-artifact output directory. Six
/// files total; all six are byte-stable across two emits.
fn run_s_perf_2_receipts_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-2-receipts-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let pipeline = seed_baseline_layer_a_pipeline();
    let residency = seed_baseline_layer_a_residency_receipt();
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let paths_and_bodies: [(String, String); 6] = [
        (
            format!("{out_dir}/s_perf_2_layer_a_resident_pipeline_v1.txt"),
            render_layer_a_resident_pipeline_text(&pipeline),
        ),
        (
            format!("{out_dir}/s_perf_2_layer_a_resident_pipeline_v1.json"),
            render_layer_a_resident_pipeline_json(&pipeline),
        ),
        (
            format!("{out_dir}/s_perf_2_layer_a_device_residency_receipt_v1.txt"),
            render_layer_a_device_residency_receipt_text(&residency),
        ),
        (
            format!("{out_dir}/s_perf_2_layer_a_device_residency_receipt_v1.json"),
            render_layer_a_device_residency_receipt_json(&residency),
        ),
        (
            format!("{out_dir}/s_perf_2_layer_a_traffic_receipt_v1.txt"),
            render_layer_a_traffic_receipt_text(&traffic),
        ),
        (
            format!("{out_dir}/s_perf_2_layer_a_traffic_receipt_v1.json"),
            render_layer_a_traffic_receipt_json(&traffic),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-3-bundle` --- render the S-PERF.3 baseline
/// public-data saturation bundle (text by default, JSON via
/// `--json`). The baseline ships five citation-only manifests
/// covering the five panel-named dataset classes (TADBench /
/// Defects4J / ADBench subset / TSB-UAD / NASA C-MAPSS).
fn run_s_perf_3_bundle(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-3-bundle`");
                return ExitCode::from(1);
            }
        }
    }
    let b = seed_baseline_public_data_saturation_bundle();
    let body = if json_mode {
        render_public_data_saturation_bundle_json(&b)
    } else {
        render_public_data_saturation_bundle_text(&b)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-3-policy` --- render the panel-locked dataset
/// materialization policy.
fn run_s_perf_3_policy(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-3-policy`");
                return ExitCode::from(1);
            }
        }
    }
    let p = build_panel_locked_dataset_materialization_policy();
    let body = if json_mode {
        render_dataset_materialization_policy_json(&p)
    } else {
        render_dataset_materialization_policy_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-3-bundle-emit` --- write the baseline bundle as
/// text + JSON, plus the materialization policy as text +
/// JSON, into the bulk-artifact output directory. Four files
/// total; all four are byte-stable across two emits.
fn run_s_perf_3_bundle_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-3-bundle-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let bundle = seed_baseline_public_data_saturation_bundle();
    let policy = build_panel_locked_dataset_materialization_policy();
    let paths_and_bodies: [(String, String); 4] = [
        (
            format!("{out_dir}/s_perf_3_public_data_saturation_bundle_v1.txt"),
            render_public_data_saturation_bundle_text(&bundle),
        ),
        (
            format!("{out_dir}/s_perf_3_public_data_saturation_bundle_v1.json"),
            render_public_data_saturation_bundle_json(&bundle),
        ),
        (
            format!("{out_dir}/s_perf_3_dataset_materialization_policy_v1.txt"),
            render_dataset_materialization_policy_text(&policy),
        ),
        (
            format!("{out_dir}/s_perf_3_dataset_materialization_policy_v1.json"),
            render_dataset_materialization_policy_json(&policy),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-4-plan` --- render the baseline active-family
/// compaction plan (text by default, JSON via `--json`).
fn run_s_perf_4_plan(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-4-plan`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_baseline_active_family_compaction_plan();
    let body = if json_mode {
        render_active_family_compaction_plan_json(&p)
    } else {
        render_active_family_compaction_plan_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-4-parameter-table-receipt` --- render the baseline
/// parameter-table receipt.
fn run_s_perf_4_parameter_table_receipt(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `s-perf-4-parameter-table-receipt`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_baseline_compacted_parameter_table_receipt();
    let body = if json_mode {
        render_compacted_parameter_table_receipt_json(&r)
    } else {
        render_compacted_parameter_table_receipt_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-4-schema` --- render the baseline benchmark schema
/// (the top-level META-hash envelope).
fn run_s_perf_4_schema(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-4-schema`");
                return ExitCode::from(1);
            }
        }
    }
    let s = seed_baseline_family_compaction_benchmark_schema();
    let body = if json_mode {
        render_family_compaction_benchmark_schema_json(&s)
    } else {
        render_family_compaction_benchmark_schema_text(&s)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-4-receipts-emit` --- write the baseline plan,
/// parameter-table receipt, and benchmark schema (each as
/// text + JSON) into the bulk-artifact output directory.
/// Six files total; all six byte-stable across two emits.
fn run_s_perf_4_receipts_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-4-receipts-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let plan = seed_baseline_active_family_compaction_plan();
    let receipt = seed_baseline_compacted_parameter_table_receipt();
    let schema = seed_baseline_family_compaction_benchmark_schema();
    let paths_and_bodies: [(String, String); 6] = [
        (
            format!("{out_dir}/s_perf_4_active_family_compaction_plan_v1.txt"),
            render_active_family_compaction_plan_text(&plan),
        ),
        (
            format!("{out_dir}/s_perf_4_active_family_compaction_plan_v1.json"),
            render_active_family_compaction_plan_json(&plan),
        ),
        (
            format!("{out_dir}/s_perf_4_compacted_parameter_table_receipt_v1.txt"),
            render_compacted_parameter_table_receipt_text(&receipt),
        ),
        (
            format!("{out_dir}/s_perf_4_compacted_parameter_table_receipt_v1.json"),
            render_compacted_parameter_table_receipt_json(&receipt),
        ),
        (
            format!("{out_dir}/s_perf_4_family_compaction_benchmark_schema_v1.txt"),
            render_family_compaction_benchmark_schema_text(&schema),
        ),
        (
            format!("{out_dir}/s_perf_4_family_compaction_benchmark_schema_v1.json"),
            render_family_compaction_benchmark_schema_json(&schema),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-5-measurement` --- render the baseline Layer-A
/// bandwidth measurement (text by default, JSON via `--json`).
fn run_s_perf_5_measurement(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-5-measurement`");
                return ExitCode::from(1);
            }
        }
    }
    let m = seed_baseline_layer_a_bandwidth_measurement();
    let body = if json_mode {
        render_layer_a_bandwidth_measurement_json(&m)
    } else {
        render_layer_a_bandwidth_measurement_text(&m)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-5-admission` --- render the baseline bandwidth-claim
/// admission verdict.
fn run_s_perf_5_admission(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-5-admission`");
                return ExitCode::from(1);
            }
        }
    }
    let a = seed_baseline_bandwidth_claim_admission();
    let body = if json_mode {
        render_bandwidth_claim_admission_json(&a)
    } else {
        render_bandwidth_claim_admission_text(&a)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-5-report` --- render the baseline effective-
/// bandwidth report (the top-level META-hash envelope binding
/// the measurement + admission + four upstream anchor hashes).
fn run_s_perf_5_report(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-5-report`");
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_baseline_effective_bandwidth_report();
    let body = if json_mode {
        render_effective_bandwidth_report_json(&r)
    } else {
        render_effective_bandwidth_report_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-5-receipts-emit` --- write the baseline measurement,
/// admission, and report (each as text + JSON) into the bulk-
/// artifact output directory. Six files total; all six byte-
/// stable across two emits.
fn run_s_perf_5_receipts_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-5-receipts-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let measurement = seed_baseline_layer_a_bandwidth_measurement();
    let admission = seed_baseline_bandwidth_claim_admission();
    let report = seed_baseline_effective_bandwidth_report();
    let paths_and_bodies: [(String, String); 6] = [
        (
            format!("{out_dir}/s_perf_5_layer_a_bandwidth_measurement_v1.txt"),
            render_layer_a_bandwidth_measurement_text(&measurement),
        ),
        (
            format!("{out_dir}/s_perf_5_layer_a_bandwidth_measurement_v1.json"),
            render_layer_a_bandwidth_measurement_json(&measurement),
        ),
        (
            format!("{out_dir}/s_perf_5_bandwidth_claim_admission_v1.txt"),
            render_bandwidth_claim_admission_text(&admission),
        ),
        (
            format!("{out_dir}/s_perf_5_bandwidth_claim_admission_v1.json"),
            render_bandwidth_claim_admission_json(&admission),
        ),
        (
            format!("{out_dir}/s_perf_5_effective_bandwidth_report_v1.txt"),
            render_effective_bandwidth_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_5_effective_bandwidth_report_v1.json"),
            render_effective_bandwidth_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-6-measurement` --- render the measured RTX 4080
/// SUPER CUDA pipeline record (text by default, JSON via
/// `--json`).
fn run_s_perf_6_measurement(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-6-measurement`");
                return ExitCode::from(1);
            }
        }
    }
    let p = seed_rtx4080_super_measured_cuda_pipeline();
    let body = if json_mode {
        render_rtx4080_super_measured_cuda_pipeline_json(&p)
    } else {
        render_rtx4080_super_measured_cuda_pipeline_text(&p)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-6-claim` --- render the measured bandwidth-claim
/// verdict for the RTX 4080 SUPER pipeline.
fn run_s_perf_6_claim(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-6-claim`");
                return ExitCode::from(1);
            }
        }
    }
    let c = seed_rtx4080_super_measured_bandwidth_claim();
    let body = if json_mode {
        render_rtx4080_super_measured_bandwidth_claim_json(&c)
    } else {
        render_rtx4080_super_measured_bandwidth_claim_text(&c)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-6-baseline` --- render the top-level S-PERF.6
/// measured baseline report (META-hash envelope binding the
/// measurement, the claim, four upstream anchor hashes, and
/// three R.12b episode-count pins).
fn run_s_perf_6_baseline(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-6-baseline`");
                return ExitCode::from(1);
            }
        }
    }
    let r = seed_rtx4080_super_measured_baseline_report();
    let body = if json_mode {
        render_rtx4080_super_measured_baseline_report_json(&r)
    } else {
        render_rtx4080_super_measured_baseline_report_text(&r)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-6-receipts-emit` --- write the measurement,
/// claim, and baseline report (each as text + JSON) into
/// the bulk-artifact output directory. Six files total;
/// all six byte-stable across two emits.
fn run_s_perf_6_receipts_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-6-receipts-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let measurement = seed_rtx4080_super_measured_cuda_pipeline();
    let claim = seed_rtx4080_super_measured_bandwidth_claim();
    let report = seed_rtx4080_super_measured_baseline_report();
    let paths_and_bodies: [(String, String); 6] = [
        (
            format!("{out_dir}/s_perf_6_measured_cuda_pipeline_v1.txt"),
            render_rtx4080_super_measured_cuda_pipeline_text(&measurement),
        ),
        (
            format!("{out_dir}/s_perf_6_measured_cuda_pipeline_v1.json"),
            render_rtx4080_super_measured_cuda_pipeline_json(&measurement),
        ),
        (
            format!("{out_dir}/s_perf_6_measured_bandwidth_claim_v1.txt"),
            render_rtx4080_super_measured_bandwidth_claim_text(&claim),
        ),
        (
            format!("{out_dir}/s_perf_6_measured_bandwidth_claim_v1.json"),
            render_rtx4080_super_measured_bandwidth_claim_json(&claim),
        ),
        (
            format!("{out_dir}/s_perf_6_measured_baseline_report_v1.txt"),
            render_rtx4080_super_measured_baseline_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_6_measured_baseline_report_v1.json"),
            render_rtx4080_super_measured_baseline_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-7-verifier` --- parse the live source reports
/// on disk + verify the S-PERF.6 receipt matches; print the
/// verifier report (text by default, JSON via `--json`).
fn run_s_perf_7_verifier(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-7-verifier`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_source_report_import_verifier_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-7-verifier seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let errors = verify_source_reports_match_s_perf_6_baseline(&report, &baseline.measurement);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-7-verifier found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_source_report_import_verifier_report_json(&report)
    } else {
        render_source_report_import_verifier_report_text(&report)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-7-verifier-emit` --- write the verifier report
/// (text + JSON) into the bulk-artifact output directory.
/// Two files total; byte-stable across two emits.
fn run_s_perf_7_verifier_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-7-verifier-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_source_report_import_verifier_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(SeedError::ReadD64 { path, message }) => {
            eprintln!("dsfb-corpus: failed to read d64 source report `{path}`: {message}");
            return ExitCode::from(5);
        }
        Err(SeedError::ReadR12b { path, message }) => {
            eprintln!("dsfb-corpus: failed to read R.12b source report `{path}`: {message}");
            return ExitCode::from(5);
        }
        Err(SeedError::ParseD64(e)) => {
            eprintln!("dsfb-corpus: d64 parser rejected: {e:?}");
            return ExitCode::from(3);
        }
        Err(SeedError::ParseR12b(e)) => {
            eprintln!("dsfb-corpus: R.12b parser rejected: {e:?}");
            return ExitCode::from(3);
        }
    };
    let paths_and_bodies: [(String, String); 2] = [
        (
            format!("{out_dir}/s_perf_7_source_report_import_verifier_v1.txt"),
            render_source_report_import_verifier_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_7_source_report_import_verifier_v1.json"),
            render_source_report_import_verifier_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-8-batched-k` --- parse the live R.12b saturation
/// table on disk + verify the batched-K saturation receipt
/// admits; print the receipt (text by default, JSON via
/// `--json`).
fn run_s_perf_8_batched_k(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-8-batched-k`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let receipt = match seed_batched_k_saturation_receipt_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-8-batched-k seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let baseline = seed_rtx4080_super_measured_baseline_report();
    let errors = verify_batched_k_saturation_receipt(&receipt, &baseline.measurement);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-8-batched-k found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_batched_k_saturation_receipt_json(&receipt)
    } else {
        render_batched_k_saturation_receipt_text(&receipt)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-8-batched-k-emit` --- write the S-PERF.8
/// batched-K saturation receipt (text + JSON) into the bulk-
/// artifact output directory. Two files total; byte-stable
/// across two emits.
fn run_s_perf_8_batched_k_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: &str = "crates/dsfb-gpu-atlas-corpus/out";
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir = &rest[idx + 1];
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = &other["--out-dir=".len()..];
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-8-batched-k-emit`");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(err) = std::fs::create_dir_all(out_dir) {
        eprintln!("dsfb-corpus: failed to create `{out_dir}`: {err}");
        return ExitCode::from(5);
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let receipt = match seed_batched_k_saturation_receipt_from_disk(&repo_root) {
        Ok(r) => r,
        Err(SPerf8SeedError::ReadR12b { path, message }) => {
            eprintln!("dsfb-corpus: failed to read R.12b source report `{path}`: {message}");
            return ExitCode::from(5);
        }
        Err(SPerf8SeedError::Parse(e)) => {
            eprintln!("dsfb-corpus: R.12b parser rejected: {e:?}");
            return ExitCode::from(3);
        }
        Err(SPerf8SeedError::SeedSPerf7(message)) => {
            eprintln!("dsfb-corpus: S-PERF.7 seed failed: {message}");
            return ExitCode::from(5);
        }
    };
    let paths_and_bodies: [(String, String); 2] = [
        (
            format!("{out_dir}/s_perf_8_batched_k_saturation_receipt_v1.txt"),
            render_batched_k_saturation_receipt_text(&receipt),
        ),
        (
            format!("{out_dir}/s_perf_8_batched_k_saturation_receipt_v1.json"),
            render_batched_k_saturation_receipt_json(&receipt),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-10-digest-lane` --- parse the four `tree_digest`
/// stage timings from the live d64 stage-timing source
/// report + verify the DigestLanePlanV1 admits; print the
/// plan (text by default, JSON via `--json`). Exits 3 on
/// any drift, 0 on admit.
fn run_s_perf_10_digest_lane(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-10-digest-lane`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let plan = match seed_digest_lane_plan_from_disk(&repo_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-10-digest-lane seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let errors = verify_digest_lane_plan(&plan);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-10-digest-lane found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_digest_lane_plan_json(&plan)
    } else {
        render_digest_lane_plan_text(&plan)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-10-digest-lane-emit` --- write both
/// `s_perf_10_digest_lane_plan_v1.{txt,json}` byte-stable
/// to the chosen directory.
fn run_s_perf_10_digest_lane_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: String = "crates/dsfb-gpu-atlas-corpus/out".to_string();
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir.clone_from(&rest[idx + 1]);
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = other["--out-dir=".len()..].to_string();
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-10-digest-lane-emit`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let plan = match seed_digest_lane_plan_from_disk(&repo_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-10-digest-lane-emit seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let paths_and_bodies = [
        (
            format!("{out_dir}/s_perf_10_digest_lane_plan_v1.txt"),
            render_digest_lane_plan_text(&plan),
        ),
        (
            format!("{out_dir}/s_perf_10_digest_lane_plan_v1.json"),
            render_digest_lane_plan_json(&plan),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-11-digest-compaction` seeds the S-PERF.11 measured
/// digest-lane compaction receipt from the live on-disk post
/// source-report plus the live upstream anchors (S-PERF.6
/// baseline, S-PERF.7 verifier, S-PERF.8.1 batched-K, S-PERF.10
/// digest-lane plan), verifies it, and prints the receipt (text
/// by default, JSON via `--json`). Exits 3 on any drift, 0 on
/// admit.
fn run_s_perf_11_digest_compaction(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-11-digest-compaction`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_bandwidth_delta_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-11-digest-compaction seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let errors = verify_bandwidth_delta_report(&report);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-11-digest-compaction found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_bandwidth_delta_report_json(&report)
    } else {
        render_bandwidth_delta_report_text(&report)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-11-digest-compaction-emit` --- write both
/// `s_perf_11_measured_digest_compaction_report_v1.{txt,json}`
/// byte-stable to the chosen directory.
fn run_s_perf_11_digest_compaction_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: String = "crates/dsfb-gpu-atlas-corpus/out".to_string();
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir.clone_from(&rest[idx + 1]);
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = other["--out-dir=".len()..].to_string();
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `s-perf-11-digest-compaction-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_bandwidth_delta_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-11-digest-compaction-emit seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let paths_and_bodies = [
        (
            format!("{out_dir}/s_perf_11_measured_digest_compaction_report_v1.txt"),
            render_bandwidth_delta_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_11_measured_digest_compaction_report_v1.json"),
            render_bandwidth_delta_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-11-1-bottleneck-triage` seeds the S-PERF.11.1
/// post-rewrite bottleneck triage receipt from the live triage
/// source-report plus the live upstream S-PERF.11 anchor,
/// verifies it, and prints the receipt (text by default, JSON
/// via `--json`). Exits 3 on any drift, 0 on admit.
fn run_s_perf_11_1_bottleneck_triage(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `s-perf-11-1-bottleneck-triage`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_post_rewrite_bottleneck_triage_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-11-1-bottleneck-triage seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let errors = verify_post_rewrite_bottleneck_triage_report(&report);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-11-1-bottleneck-triage found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_post_rewrite_bottleneck_triage_report_json(&report)
    } else {
        render_post_rewrite_bottleneck_triage_report_text(&report)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-11-1-bottleneck-triage-emit` --- write both
/// `s_perf_11_1_bottleneck_triage_report_v1.{txt,json}`
/// byte-stable to the chosen directory.
fn run_s_perf_11_1_bottleneck_triage_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: String = "crates/dsfb-gpu-atlas-corpus/out".to_string();
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir.clone_from(&rest[idx + 1]);
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = other["--out-dir=".len()..].to_string();
                idx += 1;
            }
            other => {
                eprintln!(
                    "dsfb-corpus: unknown flag `{other}` for `s-perf-11-1-bottleneck-triage-emit`"
                );
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_post_rewrite_bottleneck_triage_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-11-1-bottleneck-triage-emit seed failed: {e:?}");
            return ExitCode::from(5);
        }
    };
    let paths_and_bodies = [
        (
            format!("{out_dir}/s_perf_11_1_bottleneck_triage_report_v1.txt"),
            render_post_rewrite_bottleneck_triage_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_11_1_bottleneck_triage_report_v1.json"),
            render_post_rewrite_bottleneck_triage_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}

/// `s-perf-12-promotion` seeds the S-PERF.12
/// CompactDensorDigestV1 throughput-mode promotion report
/// from the live pinned S-PERF.11 + S-PERF.11.1 source
/// reports + `corpus_hash_v1`, runs the verifier, and prints
/// the receipt (text by default, JSON via `--json`). Exits 3
/// on any panel-locked drift (8 campaign-identity negatives
/// plus 4 structural-defect rules); exits 0 on admit. Use
/// `--out PATH` to write the receipt to a file.
fn run_s_perf_12_promotion(rest: &[String]) -> ExitCode {
    let mut json_mode = false;
    let mut out_path: Option<String> = None;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--json" => {
                json_mode = true;
                idx += 1;
            }
            "--out" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out` requires a path argument");
                    return ExitCode::from(1);
                }
                out_path = Some(rest[idx + 1].clone());
                idx += 2;
            }
            other if other.starts_with("--out=") => {
                out_path = Some(other["--out=".len()..].to_string());
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-12-promotion`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_s_perf_12_promotion_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-12-promotion seed failed: {e}");
            return ExitCode::from(5);
        }
    };
    let errors = verify_promotion_report(&report);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-12-promotion found {} drift(s):",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let body = if json_mode {
        render_promotion_report_json(&report)
    } else {
        render_promotion_report_text(&report)
    };
    emit_or_print(out_path, &body)
}

/// `s-perf-12-promotion-emit` writes the panel-locked
/// `s_perf_12_compact_densor_digest_v1_promotion_v1.{txt,json}`
/// pair byte-stable to the chosen directory (default
/// `crates/dsfb-gpu-atlas-corpus/out`). Verifier runs first;
/// exits 3 on drift, 0 on admit.
fn run_s_perf_12_promotion_emit(rest: &[String]) -> ExitCode {
    let mut out_dir: String = "crates/dsfb-gpu-atlas-corpus/out".to_string();
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--out-dir" => {
                if idx + 1 >= rest.len() {
                    eprintln!("dsfb-corpus: `--out-dir` requires a path argument");
                    return ExitCode::from(1);
                }
                out_dir.clone_from(&rest[idx + 1]);
                idx += 2;
            }
            other if other.starts_with("--out-dir=") => {
                out_dir = other["--out-dir=".len()..].to_string();
                idx += 1;
            }
            other => {
                eprintln!("dsfb-corpus: unknown flag `{other}` for `s-perf-12-promotion-emit`");
                return ExitCode::from(1);
            }
        }
    }
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
    let report = match seed_s_perf_12_promotion_report_from_disk(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("dsfb-corpus: s-perf-12-promotion-emit seed failed: {e}");
            return ExitCode::from(5);
        }
    };
    let errors = verify_promotion_report(&report);
    if !errors.is_empty() {
        eprintln!(
            "dsfb-corpus: s-perf-12-promotion-emit found {} drift(s); refusing to write artifacts:",
            errors.len()
        );
        for e in &errors {
            eprintln!("  {e:?}");
        }
        return ExitCode::from(3);
    }
    let paths_and_bodies = [
        (
            format!("{out_dir}/s_perf_12_compact_densor_digest_v1_promotion_v1.txt"),
            render_promotion_report_text(&report),
        ),
        (
            format!("{out_dir}/s_perf_12_compact_densor_digest_v1_promotion_v1.json"),
            render_promotion_report_json(&report),
        ),
    ];
    for (path, body) in &paths_and_bodies {
        if let Err(err) = std::fs::write(path, body) {
            eprintln!("dsfb-corpus: failed to write `{path}`: {err}");
            return ExitCode::from(5);
        }
        println!("wrote {path} ({} bytes)", body.len());
    }
    ExitCode::SUCCESS
}
