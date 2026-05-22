//! S1.3g acceptance suite --- OTel binding receipt
//! invariants.
//!
//! Ten panel-required load-bearing negatives pin the
//! discipline S1.3g exists to prove:
//!
//! * `s13g_rejects_binding_without_timestamp_law`
//! * `s13g_rejects_metric_binding_without_unit_or_temporality_law`
//! * `s13g_rejects_span_binding_without_trace_or_span_identity_law`
//! * `s13g_rejects_log_binding_without_body_hash_or_severity_law`
//! * `s13g_rejects_resource_binding_without_resource_identity_law`
//! * `s13g_rejects_binding_that_claims_live_ingestion`
//! * `s13g_rejects_binding_that_depends_on_otel_sdk_runtime`
//! * `s13g_rejects_stale_s13a_otel_binding_references`
//! * `s13g_rejects_nondeterministic_attribute_ordering`
//! * `s13g_rejects_binding_without_evidence_densor_field_mapping`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > S1.3f binds court authority into CaseFileV2; S1.3g
//! > defines how external OTel telemetry can be mapped into
//! > EvidenceDensor fields without yet ingesting it.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::s1_3g_otel_binding::{
    build_otel_binding_receipt, build_otel_binding_receipt_from, default_log_binding,
    default_metric_binding, default_resource_binding, default_span_binding,
    forbidden_live_ingestion_substrings, forbidden_sdk_runtime_substrings,
    forbidden_stale_s13a_substrings, render_log_binding_json, render_log_binding_text,
    render_metric_binding_json, render_metric_binding_text, render_otel_binding_receipt_json,
    render_otel_binding_receipt_text, render_resource_binding_json, render_resource_binding_text,
    render_span_binding_json, render_span_binding_text, required_attribute_ordering_substring,
    verify_otel_binding_receipt, OTelBindingReceiptTypesV1, S13gVerifyErrorKind,
    OTEL_BINDING_RECEIPT_DOMAIN_V1, OTEL_BINDING_RECEIPT_SCHEMA_V1, OTEL_LOG_BINDING_DOMAIN_V1,
    OTEL_LOG_BINDING_SCHEMA_V1, OTEL_METRIC_BINDING_DOMAIN_V1, OTEL_METRIC_BINDING_SCHEMA_V1,
    OTEL_RESOURCE_BINDING_DOMAIN_V1, OTEL_RESOURCE_BINDING_SCHEMA_V1, OTEL_SPAN_BINDING_DOMAIN_V1,
    OTEL_SPAN_BINDING_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn fresh_receipt() -> OTelBindingReceiptTypesV1 {
    build_otel_binding_receipt()
}

fn verify_default() -> Vec<dsfb_gpu_atlas_corpus::s1_3g_otel_binding::S13gVerifyError> {
    verify_otel_binding_receipt(&fresh_receipt())
}

// ---------------------------------------------------------------
// Baseline state
// ---------------------------------------------------------------

#[test]
fn s13g_default_receipt_passes_verifier() {
    let errors = verify_default();
    assert!(
        errors.is_empty(),
        "expected zero verifier errors at S1.3g baseline; got {errors:?}"
    );
}

#[test]
fn s13g_seed_len_pinned_at_54() {
    assert_eq!(SEED.len(), 54);
}

#[test]
fn s13g_receipt_pins_corpus_hash_v1_live_value() {
    let r = fresh_receipt();
    assert_eq!(r.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn s13g_receipt_pins_seed_len_54() {
    assert_eq!(fresh_receipt().seed_len, 54);
}

// ---------------------------------------------------------------
// Determinism + sensitivity invariants
// ---------------------------------------------------------------

#[test]
fn s13g_receipt_hash_is_deterministic_across_two_builds() {
    let a = build_otel_binding_receipt().otel_binding_receipt_hash_v1;
    let b = build_otel_binding_receipt().otel_binding_receipt_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13g_per_signal_binding_hashes_are_deterministic_across_two_builds() {
    let a = build_otel_binding_receipt();
    let b = build_otel_binding_receipt();
    assert_eq!(
        a.span_binding.otel_span_binding_hash_v1,
        b.span_binding.otel_span_binding_hash_v1
    );
    assert_eq!(
        a.metric_binding.otel_metric_binding_hash_v1,
        b.metric_binding.otel_metric_binding_hash_v1
    );
    assert_eq!(
        a.log_binding.otel_log_binding_hash_v1,
        b.log_binding.otel_log_binding_hash_v1
    );
    assert_eq!(
        a.resource_binding.otel_resource_binding_hash_v1,
        b.resource_binding.otel_resource_binding_hash_v1
    );
}

#[test]
fn s13g_receipt_text_is_byte_stable_across_two_renders() {
    let a = render_otel_binding_receipt_text(&fresh_receipt());
    let b = render_otel_binding_receipt_text(&fresh_receipt());
    assert_eq!(a, b);
}

#[test]
fn s13g_receipt_json_is_byte_stable_across_two_renders() {
    let a = render_otel_binding_receipt_json(&fresh_receipt());
    let b = render_otel_binding_receipt_json(&fresh_receipt());
    assert_eq!(a, b);
}

#[test]
fn s13g_span_binding_text_is_byte_stable() {
    let a = render_span_binding_text(&fresh_receipt().span_binding);
    let b = render_span_binding_text(&fresh_receipt().span_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_metric_binding_text_is_byte_stable() {
    let a = render_metric_binding_text(&fresh_receipt().metric_binding);
    let b = render_metric_binding_text(&fresh_receipt().metric_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_log_binding_text_is_byte_stable() {
    let a = render_log_binding_text(&fresh_receipt().log_binding);
    let b = render_log_binding_text(&fresh_receipt().log_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_resource_binding_text_is_byte_stable() {
    let a = render_resource_binding_text(&fresh_receipt().resource_binding);
    let b = render_resource_binding_text(&fresh_receipt().resource_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_span_binding_json_is_byte_stable() {
    let a = render_span_binding_json(&fresh_receipt().span_binding);
    let b = render_span_binding_json(&fresh_receipt().span_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_metric_binding_json_is_byte_stable() {
    let a = render_metric_binding_json(&fresh_receipt().metric_binding);
    let b = render_metric_binding_json(&fresh_receipt().metric_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_log_binding_json_is_byte_stable() {
    let a = render_log_binding_json(&fresh_receipt().log_binding);
    let b = render_log_binding_json(&fresh_receipt().log_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_resource_binding_json_is_byte_stable() {
    let a = render_resource_binding_json(&fresh_receipt().resource_binding);
    let b = render_resource_binding_json(&fresh_receipt().resource_binding);
    assert_eq!(a, b);
}

#[test]
fn s13g_receipt_hash_changes_when_a_binding_law_string_changes() {
    let base = build_otel_binding_receipt().otel_binding_receipt_hash_v1;
    let mut metric = default_metric_binding();
    metric.unit_law = "UCUM unit codes; canonical lowercase; sorted ascending by attribute key";
    // Force per-binding hash recompute by re-emitting through
    // the from-helper so the new wire bytes get hashed.
    let mutated = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    // The metric binding's bytes changed → its hash changed →
    // top-level receipt hash changed.
    assert_ne!(base, mutated.otel_binding_receipt_hash_v1);
}

// ---------------------------------------------------------------
// Domain-separator + schema-id pins
// ---------------------------------------------------------------

#[test]
fn s13g_span_binding_domain_is_pinned() {
    assert_eq!(
        OTEL_SPAN_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0"
    );
}

#[test]
fn s13g_span_binding_schema_id_is_pinned() {
    assert_eq!(
        OTEL_SPAN_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1"
    );
}

#[test]
fn s13g_metric_binding_domain_is_pinned() {
    assert_eq!(
        OTEL_METRIC_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0"
    );
}

#[test]
fn s13g_metric_binding_schema_id_is_pinned() {
    assert_eq!(
        OTEL_METRIC_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1"
    );
}

#[test]
fn s13g_log_binding_domain_is_pinned() {
    assert_eq!(
        OTEL_LOG_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0"
    );
}

#[test]
fn s13g_log_binding_schema_id_is_pinned() {
    assert_eq!(
        OTEL_LOG_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1"
    );
}

#[test]
fn s13g_resource_binding_domain_is_pinned() {
    assert_eq!(
        OTEL_RESOURCE_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0"
    );
}

#[test]
fn s13g_resource_binding_schema_id_is_pinned() {
    assert_eq!(
        OTEL_RESOURCE_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1"
    );
}

#[test]
fn s13g_receipt_domain_is_pinned() {
    assert_eq!(
        OTEL_BINDING_RECEIPT_DOMAIN_V1,
        "DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0"
    );
}

#[test]
fn s13g_receipt_schema_id_is_pinned() {
    assert_eq!(
        OTEL_BINDING_RECEIPT_SCHEMA_V1,
        "DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1"
    );
}

#[test]
fn s13g_five_hash_namespaces_are_distinct() {
    let r = fresh_receipt();
    let hashes = [
        r.span_binding.otel_span_binding_hash_v1,
        r.metric_binding.otel_metric_binding_hash_v1,
        r.log_binding.otel_log_binding_hash_v1,
        r.resource_binding.otel_resource_binding_hash_v1,
        r.otel_binding_receipt_hash_v1,
    ];
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(hashes[i], hashes[j], "collision at ({i}, {j})");
        }
    }
}

// ---------------------------------------------------------------
// Default-state structural assertions
// ---------------------------------------------------------------

#[test]
fn s13g_default_admits_no_live_ingestion_on_any_binding() {
    let r = fresh_receipt();
    assert!(!r.span_binding.admits_live_ingestion);
    assert!(!r.metric_binding.admits_live_ingestion);
    assert!(!r.log_binding.admits_live_ingestion);
    assert!(!r.resource_binding.admits_live_ingestion);
}

#[test]
fn s13g_default_does_not_depend_on_otel_sdk_runtime_on_any_binding() {
    let r = fresh_receipt();
    assert!(!r.span_binding.depends_on_otel_sdk_runtime);
    assert!(!r.metric_binding.depends_on_otel_sdk_runtime);
    assert!(!r.log_binding.depends_on_otel_sdk_runtime);
    assert!(!r.resource_binding.depends_on_otel_sdk_runtime);
}

#[test]
fn s13g_default_evidence_densor_field_counts_match_panel_expectation() {
    let r = fresh_receipt();
    assert_eq!(r.span_binding.evidence_densor_fields.len(), 8);
    assert_eq!(r.metric_binding.evidence_densor_fields.len(), 5);
    assert_eq!(r.log_binding.evidence_densor_fields.len(), 3);
    assert_eq!(r.resource_binding.evidence_densor_fields.len(), 5);
}

#[test]
fn s13g_default_law_strings_contain_required_attribute_ordering_substring() {
    let r = fresh_receipt();
    let needed = required_attribute_ordering_substring();
    assert!(r.span_binding.attribute_ordering_law.contains(needed));
    assert!(r.metric_binding.attribute_ordering_law.contains(needed));
    assert!(r.log_binding.attribute_ordering_law.contains(needed));
    assert!(r.resource_binding.attribute_ordering_law.contains(needed));
}

#[test]
fn s13g_default_law_strings_carry_no_live_ingestion_substrings() {
    let r = fresh_receipt();
    let bad = forbidden_live_ingestion_substrings();
    let all_laws: [&str; 8] = [
        r.span_binding.trace_identity_law,
        r.span_binding.timestamp_law,
        r.metric_binding.unit_law,
        r.metric_binding.temporality_law,
        r.log_binding.body_hash_law,
        r.log_binding.severity_law,
        r.resource_binding.resource_identity_law,
        r.resource_binding.timestamp_law,
    ];
    for law in all_laws {
        for &needle in bad {
            assert!(
                !law.to_ascii_lowercase().contains(needle),
                "default law `{law}` carries forbidden live-ingestion substring `{needle}`"
            );
        }
    }
}

#[test]
fn s13g_default_law_strings_carry_no_sdk_runtime_substrings() {
    let r = fresh_receipt();
    let bad = forbidden_sdk_runtime_substrings();
    let all_laws: [&str; 4] = [
        r.span_binding.timestamp_law,
        r.metric_binding.temporality_law,
        r.log_binding.severity_law,
        r.resource_binding.resource_identity_law,
    ];
    for law in all_laws {
        for &needle in bad {
            assert!(
                !law.to_ascii_lowercase().contains(needle),
                "default law `{law}` carries forbidden SDK-runtime substring `{needle}`"
            );
        }
    }
}

#[test]
fn s13g_default_law_strings_carry_no_stale_s13a_substrings() {
    let r = fresh_receipt();
    let bad = forbidden_stale_s13a_substrings();
    let all_laws: [&str; 4] = [
        r.span_binding.timestamp_law,
        r.metric_binding.metric_name_law,
        r.log_binding.timestamp_law,
        r.resource_binding.resource_identity_law,
    ];
    for law in all_laws {
        for &needle in bad {
            assert!(
                !law.to_ascii_lowercase().contains(needle),
                "default law `{law}` carries forbidden stale-S1.3a substring `{needle}`"
            );
        }
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negatives (10)
// ---------------------------------------------------------------

#[test]
fn s13g_rejects_binding_without_timestamp_law_on_span() {
    let mut span = default_span_binding();
    span.timestamp_law = "";
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingWithoutTimestampLaw { binding_wire_name }
        if binding_wire_name == "span"
    )));
}

#[test]
fn s13g_rejects_binding_without_timestamp_law_on_metric() {
    let mut metric = default_metric_binding();
    metric.timestamp_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingWithoutTimestampLaw { binding_wire_name }
        if binding_wire_name == "metric"
    )));
}

#[test]
fn s13g_rejects_binding_without_timestamp_law_on_log() {
    let mut log = default_log_binding();
    log.timestamp_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        log,
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingWithoutTimestampLaw { binding_wire_name }
        if binding_wire_name == "log"
    )));
}

#[test]
fn s13g_rejects_binding_without_timestamp_law_on_resource() {
    let mut res = default_resource_binding();
    res.timestamp_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        default_log_binding(),
        res,
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingWithoutTimestampLaw { binding_wire_name }
        if binding_wire_name == "resource"
    )));
}

#[test]
fn s13g_rejects_metric_binding_without_unit_or_temporality_law_unit() {
    let mut metric = default_metric_binding();
    metric.unit_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::MetricBindingWithoutUnitOrTemporalityLaw { missing_law_wire_name }
        if missing_law_wire_name == "unit"
    )));
}

#[test]
fn s13g_rejects_metric_binding_without_unit_or_temporality_law_temporality() {
    let mut metric = default_metric_binding();
    metric.temporality_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::MetricBindingWithoutUnitOrTemporalityLaw { missing_law_wire_name }
        if missing_law_wire_name == "temporality"
    )));
}

#[test]
fn s13g_rejects_span_binding_without_trace_or_span_identity_law() {
    let mut span = default_span_binding();
    span.trace_identity_law = "";
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::SpanBindingWithoutTraceOrSpanIdentityLaw
    )));
}

#[test]
fn s13g_rejects_log_binding_without_body_hash_or_severity_law_body_hash() {
    let mut log = default_log_binding();
    log.body_hash_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        log,
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::LogBindingWithoutBodyHashOrSeverityLaw { missing_law_wire_name }
        if missing_law_wire_name == "body_hash"
    )));
}

#[test]
fn s13g_rejects_log_binding_without_body_hash_or_severity_law_severity() {
    let mut log = default_log_binding();
    log.severity_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        log,
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::LogBindingWithoutBodyHashOrSeverityLaw { missing_law_wire_name }
        if missing_law_wire_name == "severity"
    )));
}

#[test]
fn s13g_rejects_resource_binding_without_resource_identity_law() {
    let mut res = default_resource_binding();
    res.resource_identity_law = "";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        default_log_binding(),
        res,
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::ResourceBindingWithoutResourceIdentityLaw
    )));
}

#[test]
fn s13g_rejects_binding_that_claims_live_ingestion_via_flag() {
    let mut span = default_span_binding();
    span.admits_live_ingestion = true;
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingThatClaimsLiveIngestion { binding_wire_name, .. }
        if binding_wire_name == "span"
    )));
}

#[test]
fn s13g_rejects_binding_that_claims_live_ingestion_via_substring() {
    let mut log = default_log_binding();
    log.severity_law =
        "SeverityNumber 1-24 plus an ad-hoc LIVE INGESTION fallback hook (forbidden)";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        log,
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingThatClaimsLiveIngestion { binding_wire_name, .. }
        if binding_wire_name == "log"
    )));
}

#[test]
fn s13g_rejects_binding_that_depends_on_otel_sdk_runtime_via_flag() {
    let mut metric = default_metric_binding();
    metric.depends_on_otel_sdk_runtime = true;
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingThatDependsOnOtelSdkRuntime { binding_wire_name, .. }
        if binding_wire_name == "metric"
    )));
}

#[test]
fn s13g_rejects_binding_that_depends_on_otel_sdk_runtime_via_substring() {
    let mut res = default_resource_binding();
    res.resource_identity_law =
        "service.name + instance.id; depends on OTel SDK runtime to compute (forbidden)";
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        default_log_binding(),
        res,
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingThatDependsOnOtelSdkRuntime { binding_wire_name, .. }
        if binding_wire_name == "resource"
    )));
}

#[test]
fn s13g_rejects_stale_s13a_otel_binding_references() {
    let mut span = default_span_binding();
    span.timestamp_law =
        "UTC nanoseconds; superseded by S1.3a OTel binding (rename-discipline violation)";
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::StaleS13aOtelBindingReference { .. }
    )));
}

#[test]
fn s13g_rejects_nondeterministic_attribute_ordering_on_span() {
    let mut span = default_span_binding();
    span.attribute_ordering_law = "OTel span attributes are mapped in hash-map iteration order";
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::NondeterministicAttributeOrdering { binding_wire_name }
        if binding_wire_name == "span"
    )));
}

#[test]
fn s13g_rejects_binding_without_evidence_densor_field_mapping() {
    let mut metric = default_metric_binding();
    metric.evidence_densor_fields = &[];
    let r = build_otel_binding_receipt_from(
        default_span_binding(),
        metric,
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingWithoutEvidenceDensorFieldMapping { binding_wire_name }
        if binding_wire_name == "metric"
    )));
}

// ---------------------------------------------------------------
// Structural defect rules
// ---------------------------------------------------------------

#[test]
fn s13g_rejects_binding_id_empty_on_span() {
    let mut span = default_span_binding();
    span.binding_id = "";
    let r = build_otel_binding_receipt_from(
        span,
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    );
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13gVerifyErrorKind::BindingIdEmpty { binding_wire_name }
        if binding_wire_name == "span"
    )));
}

#[test]
fn s13g_rejects_corpus_hash_v1_mismatch() {
    let mut r = fresh_receipt();
    r.corpus_hash_v1[0] ^= 0xFF;
    let errors = verify_otel_binding_receipt(&r);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13gVerifyErrorKind::CorpusHashV1Mismatch { .. })));
}

// ---------------------------------------------------------------
// Upstream anchor invariance witnesses
// ---------------------------------------------------------------

#[test]
fn s13g_does_not_alter_corpus_hash_v1() {
    let _ = build_otel_binding_receipt();
    let v1 = compute_corpus_hash_v1().bytes;
    let prefix: [u8; 4] = [0x35, 0xc2, 0x76, 0xc7];
    assert_eq!(&v1[..4], &prefix);
}

#[test]
fn s13g_does_not_alter_seed_len() {
    let _ = build_otel_binding_receipt();
    assert_eq!(SEED.len(), 54);
}

// ---------------------------------------------------------------
// Renderer-coverage spot checks
// ---------------------------------------------------------------

#[test]
fn s13g_receipt_text_contains_all_five_hash_lines() {
    let s = render_otel_binding_receipt_text(&fresh_receipt());
    assert!(s.contains("otel_span_binding_hash_v1"));
    assert!(s.contains("otel_metric_binding_hash_v1"));
    assert!(s.contains("otel_log_binding_hash_v1"));
    assert!(s.contains("otel_resource_binding_hash_v1"));
    assert!(s.contains("otel_binding_receipt_hash_v1"));
}

#[test]
fn s13g_receipt_json_parses_as_object() {
    let s = render_otel_binding_receipt_json(&fresh_receipt());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13g_span_binding_json_parses_as_object() {
    let s = render_span_binding_json(&fresh_receipt().span_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13g_metric_binding_json_parses_as_object() {
    let s = render_metric_binding_json(&fresh_receipt().metric_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13g_log_binding_json_parses_as_object() {
    let s = render_log_binding_json(&fresh_receipt().log_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13g_resource_binding_json_parses_as_object() {
    let s = render_resource_binding_json(&fresh_receipt().resource_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

// ---------------------------------------------------------------
// Forbidden-substring set boundary tests
// ---------------------------------------------------------------

#[test]
fn s13g_forbidden_live_ingestion_set_is_non_empty_and_lowercase() {
    let set = forbidden_live_ingestion_substrings();
    assert!(!set.is_empty());
    for &s in set {
        assert_eq!(s, s.to_ascii_lowercase());
    }
}

#[test]
fn s13g_forbidden_sdk_runtime_set_is_non_empty_and_lowercase() {
    let set = forbidden_sdk_runtime_substrings();
    assert!(!set.is_empty());
    for &s in set {
        assert_eq!(s, s.to_ascii_lowercase());
    }
}

#[test]
fn s13g_forbidden_stale_s13a_set_is_non_empty_and_lowercase() {
    let set = forbidden_stale_s13a_substrings();
    assert!(!set.is_empty());
    for &s in set {
        assert_eq!(s, s.to_ascii_lowercase());
    }
}

#[test]
fn s13g_required_attribute_ordering_substring_is_non_empty() {
    assert!(!required_attribute_ordering_substring().is_empty());
}
