//! S1.3g --- `OTelBindingReceiptTypes`: deterministic
//! receipt types for mapping OpenTelemetry spans, metrics,
//! logs, and resources into `EvidenceDensor` fields.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S1.3g defines deterministic receipt types for mapping
//! > OpenTelemetry spans, metrics, logs, and resources into
//! > `EvidenceDensor` fields. It is receipt-only. It does
//! > not ingest live OTLP streams, run collectors, open
//! > sockets, depend on an OTel SDK, or claim runtime
//! > interoperability.**
//!
//! Core rule (panel-locked):
//!
//! > Mapping is not ingestion.
//! > Receipt type is not adapter.
//! > Binding schema is not telemetry collection.
//!
//! ## Why
//!
//! After S1.3a–f the activation arc is complete: the case
//! file carries the entire authority chain that determined
//! which witnesses were eligible, activated, budget-admitted,
//! packed into kernel lanes, or suppressed. The first
//! external input format the Atlas will need to admit is
//! OpenTelemetry telemetry (spans / metrics / logs /
//! resources). Without a deterministic mapping receipt, any
//! future OTel-ingest commit would have no schema to satisfy
//! and no way to prove its mapping is replayable. S1.3g lands
//! the mapping contract BEFORE any ingest commit, so every
//! future ingest must emit a receipt that matches this
//! schema.
//!
//! S1.3g is **receipt-only**: it declares the four binding
//! record types ([`SpanToEvidenceDensorBindingV1`],
//! [`MetricToEvidenceDensorBindingV1`],
//! [`LogToEvidenceDensorBindingV1`],
//! [`ResourceToEvidenceDensorBindingV1`]), the top-level
//! [`OTelBindingReceiptTypesV1`] wrapper, and a verifier that
//! rejects any binding that claims live ingestion, depends
//! on an OTel SDK runtime, omits a timestamp law, omits the
//! relevant identity laws, or carries stale `"S1.3a OTel
//! binding"` references (the rename-discipline enforcer for
//! the post-T.11h next-arc sequence).
//!
//! ## Panel-locked non-claims
//!
//! S1.3g does NOT:
//!
//! - ingest live OTLP streams;
//! - run collectors, agents, or sidecars;
//! - open sockets;
//! - depend on an OTel SDK;
//! - claim runtime interoperability with the OTel
//!   reference implementation;
//! - emit detector outputs, witness records, fusion
//!   tensors, candidate intervals, or episodes;
//! - mutate any upstream hash anchor;
//! - alter `SEED.len()` (stays at 54);
//! - change S1.3a / FF.2 / FF.3 / S1.3d / S1.3e / S1.3f
//!   court decisions;
//! - decide contraindications or challenges;
//! - modify the registry crate.
//!
//! ## Hash posture
//!
//! Five new own-namespace hashes:
//!
//! - `otel_span_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0`.
//! - `otel_metric_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0`.
//! - `otel_log_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0`.
//! - `otel_resource_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0`.
//! - `otel_binding_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0`. Top-level
//!   META-hash binding the four per-binding hashes plus the
//!   corpus authority anchors.
//!
//! ## Panel-locked verdict (one line)
//!
//! > S1.3f binds court authority into CaseFileV2; S1.3g
//! > defines how external OTel telemetry can be mapped into
//! > `EvidenceDensor` fields without yet ingesting it.

use core::fmt::Write;

use dsfb_gpu_debug_core::sha256;

use crate::corpus_hash::compute_corpus_hash_v1;
use crate::seed::SEED;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `otel_span_binding_hash_v1`.
pub const OTEL_SPAN_BINDING_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1\0";
/// Schema identifier for `otel_span_binding_hash_v1`.
pub const OTEL_SPAN_BINDING_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:OTEL-SPAN-BINDING:v1";

/// Domain separator for `otel_metric_binding_hash_v1`.
pub const OTEL_METRIC_BINDING_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1\0";
/// Schema identifier for `otel_metric_binding_hash_v1`.
pub const OTEL_METRIC_BINDING_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:OTEL-METRIC-BINDING:v1";

/// Domain separator for `otel_log_binding_hash_v1`.
pub const OTEL_LOG_BINDING_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1\0";
/// Schema identifier for `otel_log_binding_hash_v1`.
pub const OTEL_LOG_BINDING_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:OTEL-LOG-BINDING:v1";

/// Domain separator for `otel_resource_binding_hash_v1`.
pub const OTEL_RESOURCE_BINDING_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1\0";
/// Schema identifier for `otel_resource_binding_hash_v1`.
pub const OTEL_RESOURCE_BINDING_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:OTEL-RESOURCE-BINDING:v1";

/// Domain separator for `otel_binding_receipt_hash_v1`.
pub const OTEL_BINDING_RECEIPT_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1\0";
/// Schema identifier for `otel_binding_receipt_hash_v1`.
pub const OTEL_BINDING_RECEIPT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:OTEL-BINDING-RECEIPT:v1";

// ---------------------------------------------------------------
// Forbidden-substring sets (panel-locked)
// ---------------------------------------------------------------

/// Phrases a binding's law strings must NOT contain. Caught
/// by the R.6 verifier rule. S1.3g is receipt-only; any law
/// string claiming live ingestion violates the core rule
/// "mapping is not ingestion".
const S13G_FORBIDDEN_LIVE_INGESTION_SUBSTRINGS: &[&str] = &[
    "live ingestion",
    "live ingest",
    "live otlp",
    "live collector",
    "live stream",
    "opens a socket",
    "opens socket",
    "runtime collector",
    "active collection",
    "ingesting now",
];

/// Phrases a binding's law strings must NOT contain. Caught
/// by the R.7 verifier rule. S1.3g declares schema only; it
/// cannot depend on the OTel SDK runtime to compute its
/// hashes.
const S13G_FORBIDDEN_SDK_RUNTIME_SUBSTRINGS: &[&str] = &[
    "otel sdk runtime",
    "opentelemetry sdk runtime",
    "depends on otel sdk",
    "depends on opentelemetry sdk",
    "requires otel sdk",
    "requires opentelemetry sdk",
];

/// Phrases a binding's law strings must NOT contain. Caught
/// by the R.8 verifier rule. The post-T.11h next-arc
/// sequence renamed "S1.3a OTel binding" to "S1.3g
/// OTelBindingReceiptTypes"; any law string still naming the
/// stale S1.3a slot is a rename-discipline failure.
const S13G_FORBIDDEN_STALE_S13A_SUBSTRINGS: &[&str] = &[
    "s1.3a otel binding",
    "s1.3a otelbinding",
    "s1.3a otel-binding",
    "s1.3a-otel-binding",
];

/// The mandatory canonical wire-name fragment every
/// attribute_ordering_law string must contain. The R.9
/// verifier rule enforces this so attribute ordering is
/// pinned to a single sort discipline; a binding cannot
/// silently use insertion-order or hash-map iteration.
const S13G_REQUIRED_ATTRIBUTE_ORDERING_SUBSTRING: &str = "sorted ascending by attribute key";

// ---------------------------------------------------------------
// Span binding record
// ---------------------------------------------------------------

/// One mapping receipt from an OpenTelemetry **span** record
/// into `EvidenceDensor` fields. The receipt declares the
/// laws that govern trace + span identity, timestamps,
/// duration, service / operation naming, status + error
/// flags, and attribute ordering; it does NOT declare the
/// runtime that would compute them (S1.3g is receipt-only).
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `otel_span_binding_hash_v1`.
#[derive(Debug, Clone)]
pub struct SpanToEvidenceDensorBindingV1 {
    /// Stable wire name of the binding (e.g.
    /// `"otel_span_binding_v1"`). Non-empty.
    pub binding_id: &'static str,
    /// Operator-readable law declaring how `trace_id` and
    /// `span_id` are canonicalised into the binding's identity
    /// key (e.g. `"trace_id+span_id 16+8 bytes canonical big-
    /// endian"`). Non-empty (R.3 verifier rule rejects empty).
    pub trace_identity_law: &'static str,
    /// Operator-readable timestamp law (e.g. `"UTC nanoseconds
    /// since Unix epoch, monotonic per trace, no leap-second
    /// smear"`). Non-empty (R.1 verifier rule rejects empty).
    pub timestamp_law: &'static str,
    /// Operator-readable duration law (e.g. `"end_time -
    /// start_time, nanoseconds, saturating on overflow"`).
    pub duration_law: &'static str,
    /// Operator-readable service.name law (canonical lowercase
    /// + UTF-8 normalisation).
    pub service_name_law: &'static str,
    /// Operator-readable operation / span.name law.
    pub operation_name_law: &'static str,
    /// Operator-readable status_code law.
    pub status_code_law: &'static str,
    /// Operator-readable error-flag law.
    pub error_flag_law: &'static str,
    /// Operator-readable attribute-ordering law. MUST
    /// contain the canonical substring
    /// `"sorted ascending by attribute key"` (R.9 verifier
    /// rule).
    pub attribute_ordering_law: &'static str,
    /// Names of the `EvidenceDensor` fields this binding
    /// populates. Non-empty (R.10 verifier rule rejects empty).
    pub evidence_densor_fields: &'static [&'static str],
    /// MUST be `false` --- S1.3g is receipt-only. R.6 rejects
    /// `true`.
    pub admits_live_ingestion: bool,
    /// MUST be `false` --- S1.3g cannot depend on the OTel
    /// SDK runtime. R.7 rejects `true`.
    pub depends_on_otel_sdk_runtime: bool,
    /// `otel_span_binding_hash_v1`.
    pub otel_span_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Metric binding record
// ---------------------------------------------------------------

/// One mapping receipt from an OpenTelemetry **metric**
/// record into `EvidenceDensor` fields. Declares the laws
/// for metric naming, unit, temporality (Cumulative / Delta),
/// timestamps, and attribute ordering.
#[derive(Debug, Clone)]
pub struct MetricToEvidenceDensorBindingV1 {
    /// Stable wire name of the binding.
    pub binding_id: &'static str,
    /// Operator-readable metric-name law.
    pub metric_name_law: &'static str,
    /// Operator-readable unit law (e.g.
    /// `"UCUM unit codes, canonical lowercase, non-empty"`).
    /// Non-empty (R.2 verifier rule rejects empty).
    pub unit_law: &'static str,
    /// Operator-readable temporality law (one of
    /// `"Cumulative"`, `"Delta"`, `"Gauge"`; the law text
    /// declares the wire enum and how aggregation windows are
    /// defined). Non-empty (R.2).
    pub temporality_law: &'static str,
    /// Operator-readable timestamp law (R.1).
    pub timestamp_law: &'static str,
    /// Operator-readable attribute-ordering law (R.9).
    pub attribute_ordering_law: &'static str,
    /// Densor field names.
    pub evidence_densor_fields: &'static [&'static str],
    /// MUST be `false` (R.6).
    pub admits_live_ingestion: bool,
    /// MUST be `false` (R.7).
    pub depends_on_otel_sdk_runtime: bool,
    /// `otel_metric_binding_hash_v1`.
    pub otel_metric_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Log binding record
// ---------------------------------------------------------------

/// One mapping receipt from an OpenTelemetry **log** record
/// into `EvidenceDensor` fields. Declares laws for log
/// timestamp, severity, body hash, and attribute ordering.
/// The body MUST be hashed (canonical bytes); the binding
/// does NOT carry log bodies directly.
#[derive(Debug, Clone)]
pub struct LogToEvidenceDensorBindingV1 {
    /// Stable wire name.
    pub binding_id: &'static str,
    /// Operator-readable timestamp law (R.1).
    pub timestamp_law: &'static str,
    /// Operator-readable severity law (e.g.
    /// `"OpenTelemetry SeverityNumber enum 1-24 + SeverityText
    /// optional override"`). Non-empty (R.4).
    pub severity_law: &'static str,
    /// Operator-readable body-hash law (e.g. `"SHA-256 over
    /// canonical UTF-8 body bytes; binary bodies hashed
    /// over their raw byte sequence"`). Non-empty (R.4).
    pub body_hash_law: &'static str,
    /// Operator-readable attribute-ordering law (R.9).
    pub attribute_ordering_law: &'static str,
    /// Densor field names.
    pub evidence_densor_fields: &'static [&'static str],
    /// MUST be `false` (R.6).
    pub admits_live_ingestion: bool,
    /// MUST be `false` (R.7).
    pub depends_on_otel_sdk_runtime: bool,
    /// `otel_log_binding_hash_v1`.
    pub otel_log_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Resource binding record
// ---------------------------------------------------------------

/// One mapping receipt from an OpenTelemetry **resource**
/// record into `EvidenceDensor` fields. Declares laws for
/// resource identity (service.name + instance + version +
/// host id), timestamps (resource snapshots are time-pinned),
/// and attribute ordering.
#[derive(Debug, Clone)]
pub struct ResourceToEvidenceDensorBindingV1 {
    /// Stable wire name.
    pub binding_id: &'static str,
    /// Operator-readable resource-identity law (e.g.
    /// `"service.name + service.instance.id + service.version + host.id, canonical lowercase, sorted ascending"`).
    /// Non-empty (R.5).
    pub resource_identity_law: &'static str,
    /// Operator-readable timestamp law (R.1).
    pub timestamp_law: &'static str,
    /// Operator-readable attribute-ordering law (R.9).
    pub attribute_ordering_law: &'static str,
    /// Densor field names.
    pub evidence_densor_fields: &'static [&'static str],
    /// MUST be `false` (R.6).
    pub admits_live_ingestion: bool,
    /// MUST be `false` (R.7).
    pub depends_on_otel_sdk_runtime: bool,
    /// `otel_resource_binding_hash_v1`.
    pub otel_resource_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level receipt
// ---------------------------------------------------------------

/// The top-level S1.3g binding receipt. Wraps the four per-
/// signal bindings plus the corpus authority anchors so one
/// hash pins the entire S1.3g mapping contract.
#[derive(Debug, Clone)]
pub struct OTelBindingReceiptTypesV1 {
    /// Wrapped span binding.
    pub span_binding: SpanToEvidenceDensorBindingV1,
    /// Wrapped metric binding.
    pub metric_binding: MetricToEvidenceDensorBindingV1,
    /// Wrapped log binding.
    pub log_binding: LogToEvidenceDensorBindingV1,
    /// Wrapped resource binding.
    pub resource_binding: ResourceToEvidenceDensorBindingV1,
    /// Historical seed-corpus anchor (unchanged across the
    /// entire post-T.12.consolidate arc).
    pub corpus_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// `otel_binding_receipt_hash_v1`.
    pub otel_binding_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S1.3g rejected a binding receipt. Ten panel-required
/// load-bearing negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S13gVerifyErrorKind {
    /// Panel-required negative #1. A binding has an empty
    /// `timestamp_law`.
    BindingWithoutTimestampLaw {
        /// The binding wire name (e.g. `"span"`).
        binding_wire_name: &'static str,
    },
    /// Panel-required negative #2. The metric binding has an
    /// empty `unit_law` or `temporality_law`.
    MetricBindingWithoutUnitOrTemporalityLaw {
        /// Which field is empty (`"unit"` or
        /// `"temporality"`).
        missing_law_wire_name: &'static str,
    },
    /// Panel-required negative #3. The span binding has an
    /// empty `trace_identity_law`.
    SpanBindingWithoutTraceOrSpanIdentityLaw,
    /// Panel-required negative #4. The log binding has an
    /// empty `body_hash_law` or `severity_law`.
    LogBindingWithoutBodyHashOrSeverityLaw {
        /// Which field is empty (`"body_hash"` or
        /// `"severity"`).
        missing_law_wire_name: &'static str,
    },
    /// Panel-required negative #5. The resource binding has
    /// an empty `resource_identity_law`.
    ResourceBindingWithoutResourceIdentityLaw,
    /// Panel-required negative #6. A binding either sets
    /// `admits_live_ingestion = true` or carries a live-
    /// ingestion-claim substring inside one of its law
    /// strings.
    BindingThatClaimsLiveIngestion {
        /// The binding wire name.
        binding_wire_name: &'static str,
        /// The forbidden substring observed (empty when the
        /// `admits_live_ingestion` flag itself was true).
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #7. A binding either sets
    /// `depends_on_otel_sdk_runtime = true` or carries an
    /// SDK-runtime-claim substring inside one of its law
    /// strings.
    BindingThatDependsOnOtelSdkRuntime {
        /// The binding wire name.
        binding_wire_name: &'static str,
        /// The forbidden substring observed (empty when the
        /// `depends_on_otel_sdk_runtime` flag itself was
        /// true).
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #8. A binding's law string
    /// names the pre-rename `"S1.3a OTel binding"` slot. The
    /// post-T.11h next-arc sequence renamed the slot to
    /// `"S1.3g OTelBindingReceiptTypes"`; any stale reference
    /// is a rename-discipline failure.
    StaleS13aOtelBindingReference {
        /// The binding wire name.
        binding_wire_name: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #9. A binding's
    /// `attribute_ordering_law` does NOT contain the
    /// canonical substring `"sorted ascending by attribute
    /// key"`.
    NondeterministicAttributeOrdering {
        /// The binding wire name.
        binding_wire_name: &'static str,
    },
    /// Panel-required negative #10. A binding has an empty
    /// `evidence_densor_fields` slice.
    BindingWithoutEvidenceDensorFieldMapping {
        /// The binding wire name.
        binding_wire_name: &'static str,
    },
    /// A binding's `binding_id` is empty.
    BindingIdEmpty {
        /// The binding wire name.
        binding_wire_name: &'static str,
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
    /// `corpus_hash_v1` pinned on the receipt does not equal
    /// the live `compute_corpus_hash_v1()` result.
    CorpusHashV1Mismatch {
        /// Hash the receipt claims.
        claimed: [u8; 32],
        /// Hash the live builder returns.
        actual: [u8; 32],
    },
    /// Two binding hashes collide (a binding-level
    /// distinctness failure).
    BindingHashCollision,
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S13gVerifyError {
    /// Error kind (see [`S13gVerifyErrorKind`]).
    pub kind: S13gVerifyErrorKind,
}

// ---------------------------------------------------------------
// Default seed
// ---------------------------------------------------------------

/// Panel-locked default span binding. Every law string is a
/// canonical wire-name declaration. The verifier accepts this
/// receipt with zero errors.
#[must_use]
pub fn default_span_binding() -> SpanToEvidenceDensorBindingV1 {
    let mut b = SpanToEvidenceDensorBindingV1 {
        binding_id: "otel_span_binding_v1",
        trace_identity_law:
            "trace_id 16 bytes + span_id 8 bytes canonical big-endian pair pinned in the receipt",
        timestamp_law:
            "UTC nanoseconds since Unix epoch, monotonic per trace, no leap-second smear",
        duration_law: "end_time - start_time, nanoseconds, saturating on overflow",
        service_name_law: "OTel service.name resource attribute, canonical lowercase, NFC normalised UTF-8",
        operation_name_law: "span.name, canonical UTF-8 NFC, non-empty",
        status_code_law: "OTel StatusCode enum (Unset | Ok | Error) declared as wire names",
        error_flag_law: "boolean derived from StatusCode == Error",
        attribute_ordering_law:
            "OTel span attributes are mapped sorted ascending by attribute key under canonical UTF-8 ordering",
        evidence_densor_fields: &[
            "EvidenceDensor::trace_id",
            "EvidenceDensor::span_id",
            "EvidenceDensor::service_name",
            "EvidenceDensor::operation_name",
            "EvidenceDensor::start_time_ns",
            "EvidenceDensor::duration_ns",
            "EvidenceDensor::status_code",
            "EvidenceDensor::error_flag",
        ],
        admits_live_ingestion: false,
        depends_on_otel_sdk_runtime: false,
        otel_span_binding_hash_v1: [0u8; 32],
    };
    b.otel_span_binding_hash_v1 = compute_span_binding_hash(&b);
    b
}

/// Panel-locked default metric binding.
#[must_use]
pub fn default_metric_binding() -> MetricToEvidenceDensorBindingV1 {
    let mut b = MetricToEvidenceDensorBindingV1 {
        binding_id: "otel_metric_binding_v1",
        metric_name_law: "OTel metric name, canonical lowercase NFC UTF-8, non-empty",
        unit_law:
            "UCUM unit codes (e.g. ms, By, 1) canonical lowercase, non-empty; dimensionless declared as '1'",
        temporality_law:
            "OTel AggregationTemporality wire enum (Cumulative | Delta | Gauge); aggregation window declared in nanoseconds",
        timestamp_law:
            "UTC nanoseconds since Unix epoch, monotonic per metric stream, no leap-second smear",
        attribute_ordering_law:
            "OTel metric data point attributes are mapped sorted ascending by attribute key under canonical UTF-8 ordering",
        evidence_densor_fields: &[
            "EvidenceDensor::metric_name",
            "EvidenceDensor::metric_unit",
            "EvidenceDensor::metric_value_q",
            "EvidenceDensor::metric_temporality",
            "EvidenceDensor::metric_timestamp_ns",
        ],
        admits_live_ingestion: false,
        depends_on_otel_sdk_runtime: false,
        otel_metric_binding_hash_v1: [0u8; 32],
    };
    b.otel_metric_binding_hash_v1 = compute_metric_binding_hash(&b);
    b
}

/// Panel-locked default log binding.
#[must_use]
pub fn default_log_binding() -> LogToEvidenceDensorBindingV1 {
    let mut b = LogToEvidenceDensorBindingV1 {
        binding_id: "otel_log_binding_v1",
        timestamp_law:
            "UTC nanoseconds since Unix epoch, monotonic per log stream, no leap-second smear",
        severity_law:
            "OTel SeverityNumber 1-24 wire enum plus optional SeverityText override declared canonical lowercase",
        body_hash_law:
            "SHA-256 over canonical UTF-8 body bytes; binary bodies hashed over the raw byte sequence; the binding carries the hash, never the body",
        attribute_ordering_law:
            "OTel log record attributes are mapped sorted ascending by attribute key under canonical UTF-8 ordering",
        evidence_densor_fields: &[
            "EvidenceDensor::log_timestamp_ns",
            "EvidenceDensor::log_severity",
            "EvidenceDensor::log_body_hash",
        ],
        admits_live_ingestion: false,
        depends_on_otel_sdk_runtime: false,
        otel_log_binding_hash_v1: [0u8; 32],
    };
    b.otel_log_binding_hash_v1 = compute_log_binding_hash(&b);
    b
}

/// Panel-locked default resource binding.
#[must_use]
pub fn default_resource_binding() -> ResourceToEvidenceDensorBindingV1 {
    let mut b = ResourceToEvidenceDensorBindingV1 {
        binding_id: "otel_resource_binding_v1",
        resource_identity_law:
            "OTel resource identity = (service.name + service.instance.id + service.version + host.id) canonical lowercase, NFC UTF-8, with empty components elided",
        timestamp_law:
            "UTC nanoseconds since Unix epoch for resource snapshot capture, no leap-second smear",
        attribute_ordering_law:
            "OTel resource attributes are mapped sorted ascending by attribute key under canonical UTF-8 ordering",
        evidence_densor_fields: &[
            "EvidenceDensor::resource_service_name",
            "EvidenceDensor::resource_service_instance_id",
            "EvidenceDensor::resource_service_version",
            "EvidenceDensor::resource_host_id",
            "EvidenceDensor::resource_snapshot_timestamp_ns",
        ],
        admits_live_ingestion: false,
        depends_on_otel_sdk_runtime: false,
        otel_resource_binding_hash_v1: [0u8; 32],
    };
    b.otel_resource_binding_hash_v1 = compute_resource_binding_hash(&b);
    b
}

// ---------------------------------------------------------------
// Builder
// ---------------------------------------------------------------

/// Build the production S1.3g binding receipt from the
/// panel-locked default per-signal bindings. Two builds
/// produce byte-identical output.
#[must_use]
pub fn build_otel_binding_receipt() -> OTelBindingReceiptTypesV1 {
    build_otel_binding_receipt_from(
        default_span_binding(),
        default_metric_binding(),
        default_log_binding(),
        default_resource_binding(),
    )
}

/// Build the receipt from a fully-specified per-signal
/// binding set. Used by tests to inject mutated bindings and
/// observe verifier rejection. Each per-binding hash is
/// recomputed from the mutated binding's bytes so callers
/// cannot smuggle stale hashes through the constructor.
#[must_use]
pub fn build_otel_binding_receipt_from(
    mut span_binding: SpanToEvidenceDensorBindingV1,
    mut metric_binding: MetricToEvidenceDensorBindingV1,
    mut log_binding: LogToEvidenceDensorBindingV1,
    mut resource_binding: ResourceToEvidenceDensorBindingV1,
) -> OTelBindingReceiptTypesV1 {
    span_binding.otel_span_binding_hash_v1 = compute_span_binding_hash(&span_binding);
    metric_binding.otel_metric_binding_hash_v1 = compute_metric_binding_hash(&metric_binding);
    log_binding.otel_log_binding_hash_v1 = compute_log_binding_hash(&log_binding);
    resource_binding.otel_resource_binding_hash_v1 =
        compute_resource_binding_hash(&resource_binding);
    let mut r = OTelBindingReceiptTypesV1 {
        span_binding,
        metric_binding,
        log_binding,
        resource_binding,
        corpus_hash_v1: compute_corpus_hash_v1().bytes,
        seed_len: u32::try_from(SEED.len()).unwrap_or(u32::MAX),
        otel_binding_receipt_hash_v1: [0u8; 32],
    };
    r.otel_binding_receipt_hash_v1 = compute_receipt_hash(&r);
    r
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_span_binding_hash(b: &SpanToEvidenceDensorBindingV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(OTEL_SPAN_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(OTEL_SPAN_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, b.binding_id.as_bytes());
    push_len_prefixed(&mut buf, b.trace_identity_law.as_bytes());
    push_len_prefixed(&mut buf, b.timestamp_law.as_bytes());
    push_len_prefixed(&mut buf, b.duration_law.as_bytes());
    push_len_prefixed(&mut buf, b.service_name_law.as_bytes());
    push_len_prefixed(&mut buf, b.operation_name_law.as_bytes());
    push_len_prefixed(&mut buf, b.status_code_law.as_bytes());
    push_len_prefixed(&mut buf, b.error_flag_law.as_bytes());
    push_len_prefixed(&mut buf, b.attribute_ordering_law.as_bytes());
    push_str_slice(&mut buf, b.evidence_densor_fields);
    buf.push(u8::from(b.admits_live_ingestion));
    buf.push(u8::from(b.depends_on_otel_sdk_runtime));
    sha256(&buf)
}

fn compute_metric_binding_hash(b: &MetricToEvidenceDensorBindingV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(OTEL_METRIC_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(OTEL_METRIC_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, b.binding_id.as_bytes());
    push_len_prefixed(&mut buf, b.metric_name_law.as_bytes());
    push_len_prefixed(&mut buf, b.unit_law.as_bytes());
    push_len_prefixed(&mut buf, b.temporality_law.as_bytes());
    push_len_prefixed(&mut buf, b.timestamp_law.as_bytes());
    push_len_prefixed(&mut buf, b.attribute_ordering_law.as_bytes());
    push_str_slice(&mut buf, b.evidence_densor_fields);
    buf.push(u8::from(b.admits_live_ingestion));
    buf.push(u8::from(b.depends_on_otel_sdk_runtime));
    sha256(&buf)
}

fn compute_log_binding_hash(b: &LogToEvidenceDensorBindingV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(OTEL_LOG_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(OTEL_LOG_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, b.binding_id.as_bytes());
    push_len_prefixed(&mut buf, b.timestamp_law.as_bytes());
    push_len_prefixed(&mut buf, b.severity_law.as_bytes());
    push_len_prefixed(&mut buf, b.body_hash_law.as_bytes());
    push_len_prefixed(&mut buf, b.attribute_ordering_law.as_bytes());
    push_str_slice(&mut buf, b.evidence_densor_fields);
    buf.push(u8::from(b.admits_live_ingestion));
    buf.push(u8::from(b.depends_on_otel_sdk_runtime));
    sha256(&buf)
}

fn compute_resource_binding_hash(b: &ResourceToEvidenceDensorBindingV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(OTEL_RESOURCE_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(OTEL_RESOURCE_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, b.binding_id.as_bytes());
    push_len_prefixed(&mut buf, b.resource_identity_law.as_bytes());
    push_len_prefixed(&mut buf, b.timestamp_law.as_bytes());
    push_len_prefixed(&mut buf, b.attribute_ordering_law.as_bytes());
    push_str_slice(&mut buf, b.evidence_densor_fields);
    buf.push(u8::from(b.admits_live_ingestion));
    buf.push(u8::from(b.depends_on_otel_sdk_runtime));
    sha256(&buf)
}

fn compute_receipt_hash(r: &OTelBindingReceiptTypesV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(OTEL_BINDING_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(OTEL_BINDING_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&r.corpus_hash_v1);
    buf.extend_from_slice(&r.seed_len.to_be_bytes());
    buf.extend_from_slice(&r.span_binding.otel_span_binding_hash_v1);
    buf.extend_from_slice(&r.metric_binding.otel_metric_binding_hash_v1);
    buf.extend_from_slice(&r.log_binding.otel_log_binding_hash_v1);
    buf.extend_from_slice(&r.resource_binding.otel_resource_binding_hash_v1);
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn push_str_slice(buf: &mut Vec<u8>, slice: &[&str]) {
    let len = u32::try_from(slice.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    for s in slice {
        push_len_prefixed(buf, s.as_bytes());
    }
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify an S1.3g binding receipt. Returns a vector of
/// errors (empty when the receipt satisfies every panel-
/// required + structural rule).
//
// 13 rules across 4 bindings; each rule contributes a small
// constant number of lines but the aggregate exceeds the
// workspace default 100-line clippy cap. Splitting per-
// binding would obscure the rule numbering, so we accept the
// length deliberately.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_otel_binding_receipt(r: &OTelBindingReceiptTypesV1) -> Vec<S13gVerifyError> {
    let mut errors: Vec<S13gVerifyError> = Vec::new();

    // R.1 BindingWithoutTimestampLaw (applies to all four).
    if r.span_binding.timestamp_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingWithoutTimestampLaw {
                binding_wire_name: "span",
            },
        });
    }
    if r.metric_binding.timestamp_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingWithoutTimestampLaw {
                binding_wire_name: "metric",
            },
        });
    }
    if r.log_binding.timestamp_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingWithoutTimestampLaw {
                binding_wire_name: "log",
            },
        });
    }
    if r.resource_binding.timestamp_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingWithoutTimestampLaw {
                binding_wire_name: "resource",
            },
        });
    }

    // R.2 MetricBindingWithoutUnitOrTemporalityLaw.
    if r.metric_binding.unit_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::MetricBindingWithoutUnitOrTemporalityLaw {
                missing_law_wire_name: "unit",
            },
        });
    }
    if r.metric_binding.temporality_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::MetricBindingWithoutUnitOrTemporalityLaw {
                missing_law_wire_name: "temporality",
            },
        });
    }

    // R.3 SpanBindingWithoutTraceOrSpanIdentityLaw.
    if r.span_binding.trace_identity_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::SpanBindingWithoutTraceOrSpanIdentityLaw,
        });
    }

    // R.4 LogBindingWithoutBodyHashOrSeverityLaw.
    if r.log_binding.body_hash_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::LogBindingWithoutBodyHashOrSeverityLaw {
                missing_law_wire_name: "body_hash",
            },
        });
    }
    if r.log_binding.severity_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::LogBindingWithoutBodyHashOrSeverityLaw {
                missing_law_wire_name: "severity",
            },
        });
    }

    // R.5 ResourceBindingWithoutResourceIdentityLaw.
    if r.resource_binding.resource_identity_law.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::ResourceBindingWithoutResourceIdentityLaw,
        });
    }

    // R.6 BindingThatClaimsLiveIngestion + R.7
    // BindingThatDependsOnOtelSdkRuntime + R.8
    // StaleS13aOtelBindingReference + R.9
    // NondeterministicAttributeOrdering apply to every
    // binding; iterate them.
    check_binding_flags_and_substrings(
        &mut errors,
        "span",
        r.span_binding.admits_live_ingestion,
        r.span_binding.depends_on_otel_sdk_runtime,
        &[
            r.span_binding.trace_identity_law,
            r.span_binding.timestamp_law,
            r.span_binding.duration_law,
            r.span_binding.service_name_law,
            r.span_binding.operation_name_law,
            r.span_binding.status_code_law,
            r.span_binding.error_flag_law,
            r.span_binding.attribute_ordering_law,
        ],
        r.span_binding.attribute_ordering_law,
        r.span_binding.evidence_densor_fields,
        r.span_binding.binding_id,
    );
    check_binding_flags_and_substrings(
        &mut errors,
        "metric",
        r.metric_binding.admits_live_ingestion,
        r.metric_binding.depends_on_otel_sdk_runtime,
        &[
            r.metric_binding.metric_name_law,
            r.metric_binding.unit_law,
            r.metric_binding.temporality_law,
            r.metric_binding.timestamp_law,
            r.metric_binding.attribute_ordering_law,
        ],
        r.metric_binding.attribute_ordering_law,
        r.metric_binding.evidence_densor_fields,
        r.metric_binding.binding_id,
    );
    check_binding_flags_and_substrings(
        &mut errors,
        "log",
        r.log_binding.admits_live_ingestion,
        r.log_binding.depends_on_otel_sdk_runtime,
        &[
            r.log_binding.timestamp_law,
            r.log_binding.severity_law,
            r.log_binding.body_hash_law,
            r.log_binding.attribute_ordering_law,
        ],
        r.log_binding.attribute_ordering_law,
        r.log_binding.evidence_densor_fields,
        r.log_binding.binding_id,
    );
    check_binding_flags_and_substrings(
        &mut errors,
        "resource",
        r.resource_binding.admits_live_ingestion,
        r.resource_binding.depends_on_otel_sdk_runtime,
        &[
            r.resource_binding.resource_identity_law,
            r.resource_binding.timestamp_law,
            r.resource_binding.attribute_ordering_law,
        ],
        r.resource_binding.attribute_ordering_law,
        r.resource_binding.evidence_densor_fields,
        r.resource_binding.binding_id,
    );

    // Structural defects.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if r.corpus_hash_v1 != live_v1 {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::CorpusHashV1Mismatch {
                claimed: r.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }
    let hashes = [
        r.span_binding.otel_span_binding_hash_v1,
        r.metric_binding.otel_metric_binding_hash_v1,
        r.log_binding.otel_log_binding_hash_v1,
        r.resource_binding.otel_resource_binding_hash_v1,
    ];
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            if hashes[i] == hashes[j] {
                errors.push(S13gVerifyError {
                    kind: S13gVerifyErrorKind::BindingHashCollision,
                });
                break;
            }
        }
    }

    errors
}

/// Run R.6 / R.7 / R.8 / R.9 / R.10 / BindingIdEmpty checks
/// on one binding's law strings + flags + densor fields +
/// binding id. The verifier dispatches once per binding so
/// each binding can name itself in the error payload.
#[allow(clippy::too_many_arguments)] // each argument is a panel-locked input the verifier consumes
fn check_binding_flags_and_substrings(
    errors: &mut Vec<S13gVerifyError>,
    binding_wire_name: &'static str,
    admits_live_ingestion: bool,
    depends_on_otel_sdk_runtime: bool,
    law_strings: &[&'static str],
    attribute_ordering_law: &'static str,
    evidence_densor_fields: &[&'static str],
    binding_id: &'static str,
) {
    if binding_id.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingIdEmpty { binding_wire_name },
        });
    }

    if admits_live_ingestion {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingThatClaimsLiveIngestion {
                binding_wire_name,
                forbidden_substring: "",
            },
        });
    }
    if depends_on_otel_sdk_runtime {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingThatDependsOnOtelSdkRuntime {
                binding_wire_name,
                forbidden_substring: "",
            },
        });
    }

    for law in law_strings {
        for &forbidden in S13G_FORBIDDEN_LIVE_INGESTION_SUBSTRINGS {
            if contains_ascii_case_insensitive(law, forbidden) {
                errors.push(S13gVerifyError {
                    kind: S13gVerifyErrorKind::BindingThatClaimsLiveIngestion {
                        binding_wire_name,
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
        for &forbidden in S13G_FORBIDDEN_SDK_RUNTIME_SUBSTRINGS {
            if contains_ascii_case_insensitive(law, forbidden) {
                errors.push(S13gVerifyError {
                    kind: S13gVerifyErrorKind::BindingThatDependsOnOtelSdkRuntime {
                        binding_wire_name,
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
        for &forbidden in S13G_FORBIDDEN_STALE_S13A_SUBSTRINGS {
            if contains_ascii_case_insensitive(law, forbidden) {
                errors.push(S13gVerifyError {
                    kind: S13gVerifyErrorKind::StaleS13aOtelBindingReference {
                        binding_wire_name,
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
    }

    if !contains_ascii_case_insensitive(
        attribute_ordering_law,
        S13G_REQUIRED_ATTRIBUTE_ORDERING_SUBSTRING,
    ) {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::NondeterministicAttributeOrdering { binding_wire_name },
        });
    }

    if evidence_densor_fields.is_empty() {
        errors.push(S13gVerifyError {
            kind: S13gVerifyErrorKind::BindingWithoutEvidenceDensorFieldMapping {
                binding_wire_name,
            },
        });
    }
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    for window_start in 0..=h.len() - n.len() {
        let mut ok = true;
        for i in 0..n.len() {
            if !h[window_start + i].eq_ignore_ascii_case(&n[i]) {
                ok = false;
                break;
            }
        }
        if ok {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the top-level binding receipt as deterministic text.
#[must_use]
pub fn render_otel_binding_receipt_text(r: &OTelBindingReceiptTypesV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3g OTel Binding Receipt Types (v1)");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned corpus anchors");
    let _ = writeln!(s, "  corpus_hash_v1 : {}", hex32(&r.corpus_hash_v1));
    let _ = writeln!(s, "  SEED.len()     : {}", r.seed_len);
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-signal binding hashes");
    let _ = writeln!(
        s,
        "  otel_span_binding_hash_v1     : {}",
        hex32(&r.span_binding.otel_span_binding_hash_v1)
    );
    let _ = writeln!(
        s,
        "  otel_metric_binding_hash_v1   : {}",
        hex32(&r.metric_binding.otel_metric_binding_hash_v1)
    );
    let _ = writeln!(
        s,
        "  otel_log_binding_hash_v1      : {}",
        hex32(&r.log_binding.otel_log_binding_hash_v1)
    );
    let _ = writeln!(
        s,
        "  otel_resource_binding_hash_v1 : {}",
        hex32(&r.resource_binding.otel_resource_binding_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Binding ids");
    let _ = writeln!(s, "  span     : {}", r.span_binding.binding_id);
    let _ = writeln!(s, "  metric   : {}", r.metric_binding.binding_id);
    let _ = writeln!(s, "  log      : {}", r.log_binding.binding_id);
    let _ = writeln!(s, "  resource : {}", r.resource_binding.binding_id);
    let _ = writeln!(s);
    let _ = writeln!(s, "Densor field counts");
    let _ = writeln!(
        s,
        "  span_fields     : {}",
        r.span_binding.evidence_densor_fields.len()
    );
    let _ = writeln!(
        s,
        "  metric_fields   : {}",
        r.metric_binding.evidence_densor_fields.len()
    );
    let _ = writeln!(
        s,
        "  log_fields      : {}",
        r.log_binding.evidence_densor_fields.len()
    );
    let _ = writeln!(
        s,
        "  resource_fields : {}",
        r.resource_binding.evidence_densor_fields.len()
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "otel_binding_receipt_hash_v1 : {}",
        hex32(&r.otel_binding_receipt_hash_v1)
    );
    s
}

/// Render the span binding section as deterministic text.
#[must_use]
pub fn render_span_binding_text(b: &SpanToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3g Span -> EvidenceDensor binding (v1)");
    let _ = writeln!(s, "binding_id : {}", b.binding_id);
    let _ = writeln!(s, "trace_identity_law       : {}", b.trace_identity_law);
    let _ = writeln!(s, "timestamp_law            : {}", b.timestamp_law);
    let _ = writeln!(s, "duration_law             : {}", b.duration_law);
    let _ = writeln!(s, "service_name_law         : {}", b.service_name_law);
    let _ = writeln!(s, "operation_name_law       : {}", b.operation_name_law);
    let _ = writeln!(s, "status_code_law          : {}", b.status_code_law);
    let _ = writeln!(s, "error_flag_law           : {}", b.error_flag_law);
    let _ = writeln!(s, "attribute_ordering_law   : {}", b.attribute_ordering_law);
    let _ = writeln!(
        s,
        "evidence_densor_fields   : {} entries",
        b.evidence_densor_fields.len()
    );
    let _ = writeln!(s, "admits_live_ingestion    : {}", b.admits_live_ingestion);
    let _ = writeln!(
        s,
        "depends_on_otel_sdk_runtime : {}",
        b.depends_on_otel_sdk_runtime
    );
    let _ = writeln!(
        s,
        "otel_span_binding_hash_v1: {}",
        hex32(&b.otel_span_binding_hash_v1)
    );
    s
}

/// Render the metric binding section as deterministic text.
#[must_use]
pub fn render_metric_binding_text(b: &MetricToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3g Metric -> EvidenceDensor binding (v1)");
    let _ = writeln!(s, "binding_id : {}", b.binding_id);
    let _ = writeln!(s, "metric_name_law          : {}", b.metric_name_law);
    let _ = writeln!(s, "unit_law                 : {}", b.unit_law);
    let _ = writeln!(s, "temporality_law          : {}", b.temporality_law);
    let _ = writeln!(s, "timestamp_law            : {}", b.timestamp_law);
    let _ = writeln!(s, "attribute_ordering_law   : {}", b.attribute_ordering_law);
    let _ = writeln!(
        s,
        "evidence_densor_fields   : {} entries",
        b.evidence_densor_fields.len()
    );
    let _ = writeln!(s, "admits_live_ingestion    : {}", b.admits_live_ingestion);
    let _ = writeln!(
        s,
        "depends_on_otel_sdk_runtime : {}",
        b.depends_on_otel_sdk_runtime
    );
    let _ = writeln!(
        s,
        "otel_metric_binding_hash_v1: {}",
        hex32(&b.otel_metric_binding_hash_v1)
    );
    s
}

/// Render the log binding section as deterministic text.
#[must_use]
pub fn render_log_binding_text(b: &LogToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3g Log -> EvidenceDensor binding (v1)");
    let _ = writeln!(s, "binding_id : {}", b.binding_id);
    let _ = writeln!(s, "timestamp_law            : {}", b.timestamp_law);
    let _ = writeln!(s, "severity_law             : {}", b.severity_law);
    let _ = writeln!(s, "body_hash_law            : {}", b.body_hash_law);
    let _ = writeln!(s, "attribute_ordering_law   : {}", b.attribute_ordering_law);
    let _ = writeln!(
        s,
        "evidence_densor_fields   : {} entries",
        b.evidence_densor_fields.len()
    );
    let _ = writeln!(s, "admits_live_ingestion    : {}", b.admits_live_ingestion);
    let _ = writeln!(
        s,
        "depends_on_otel_sdk_runtime : {}",
        b.depends_on_otel_sdk_runtime
    );
    let _ = writeln!(
        s,
        "otel_log_binding_hash_v1: {}",
        hex32(&b.otel_log_binding_hash_v1)
    );
    s
}

/// Render the resource binding section as deterministic text.
#[must_use]
pub fn render_resource_binding_text(b: &ResourceToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3g Resource -> EvidenceDensor binding (v1)");
    let _ = writeln!(s, "binding_id : {}", b.binding_id);
    let _ = writeln!(s, "resource_identity_law    : {}", b.resource_identity_law);
    let _ = writeln!(s, "timestamp_law            : {}", b.timestamp_law);
    let _ = writeln!(s, "attribute_ordering_law   : {}", b.attribute_ordering_law);
    let _ = writeln!(
        s,
        "evidence_densor_fields   : {} entries",
        b.evidence_densor_fields.len()
    );
    let _ = writeln!(s, "admits_live_ingestion    : {}", b.admits_live_ingestion);
    let _ = writeln!(
        s,
        "depends_on_otel_sdk_runtime : {}",
        b.depends_on_otel_sdk_runtime
    );
    let _ = writeln!(
        s,
        "otel_resource_binding_hash_v1: {}",
        hex32(&b.otel_resource_binding_hash_v1)
    );
    s
}

/// Render the top-level binding receipt as canonical JSON.
#[must_use]
pub fn render_otel_binding_receipt_json(r: &OTelBindingReceiptTypesV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", OTEL_BINDING_RECEIPT_SCHEMA_V1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v1", &r.corpus_hash_v1);
    s.push(',');
    let _ = write!(s, "\"seed_len\":{}", r.seed_len);
    s.push(',');
    json_hex(
        &mut s,
        "otel_span_binding_hash_v1",
        &r.span_binding.otel_span_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "otel_metric_binding_hash_v1",
        &r.metric_binding.otel_metric_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "otel_log_binding_hash_v1",
        &r.log_binding.otel_log_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "otel_resource_binding_hash_v1",
        &r.resource_binding.otel_resource_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "otel_binding_receipt_hash_v1",
        &r.otel_binding_receipt_hash_v1,
    );
    s.push('}');
    s
}

/// Render the span binding as canonical JSON.
#[must_use]
pub fn render_span_binding_json(b: &SpanToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", OTEL_SPAN_BINDING_SCHEMA_V1);
    s.push(',');
    json_field(&mut s, "binding_id", b.binding_id);
    s.push(',');
    json_hex(
        &mut s,
        "otel_span_binding_hash_v1",
        &b.otel_span_binding_hash_v1,
    );
    s.push('}');
    s
}

/// Render the metric binding as canonical JSON.
#[must_use]
pub fn render_metric_binding_json(b: &MetricToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", OTEL_METRIC_BINDING_SCHEMA_V1);
    s.push(',');
    json_field(&mut s, "binding_id", b.binding_id);
    s.push(',');
    json_hex(
        &mut s,
        "otel_metric_binding_hash_v1",
        &b.otel_metric_binding_hash_v1,
    );
    s.push('}');
    s
}

/// Render the log binding as canonical JSON.
#[must_use]
pub fn render_log_binding_json(b: &LogToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", OTEL_LOG_BINDING_SCHEMA_V1);
    s.push(',');
    json_field(&mut s, "binding_id", b.binding_id);
    s.push(',');
    json_hex(
        &mut s,
        "otel_log_binding_hash_v1",
        &b.otel_log_binding_hash_v1,
    );
    s.push('}');
    s
}

/// Render the resource binding as canonical JSON.
#[must_use]
pub fn render_resource_binding_json(b: &ResourceToEvidenceDensorBindingV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", OTEL_RESOURCE_BINDING_SCHEMA_V1);
    s.push(',');
    json_field(&mut s, "binding_id", b.binding_id);
    s.push(',');
    json_hex(
        &mut s,
        "otel_resource_binding_hash_v1",
        &b.otel_resource_binding_hash_v1,
    );
    s.push('}');
    s
}

fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_hex(s: &mut String, key: &str, value: &[u8; 32]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    let _ = s.write_str(&hex32(value));
    s.push('"');
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Read-only access to the panel-locked forbidden-substring
/// sets. Tests consume these to crash-confirm the seed never
/// trips a scanner.
#[doc(hidden)]
#[must_use]
pub fn forbidden_live_ingestion_substrings() -> &'static [&'static str] {
    S13G_FORBIDDEN_LIVE_INGESTION_SUBSTRINGS
}
#[doc(hidden)]
#[must_use]
pub fn forbidden_sdk_runtime_substrings() -> &'static [&'static str] {
    S13G_FORBIDDEN_SDK_RUNTIME_SUBSTRINGS
}
#[doc(hidden)]
#[must_use]
pub fn forbidden_stale_s13a_substrings() -> &'static [&'static str] {
    S13G_FORBIDDEN_STALE_S13A_SUBSTRINGS
}
#[doc(hidden)]
#[must_use]
pub fn required_attribute_ordering_substring() -> &'static str {
    S13G_REQUIRED_ATTRIBUTE_ORDERING_SUBSTRING
}
