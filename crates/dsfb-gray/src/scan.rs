//! Static source-tree scanning for DSFB heuristic provenance motifs.
//!
//! This module provides a std-only companion workflow that scans Rust source
//! trees for implementation motifs related to the DSFB heuristics bank.
//! It does not replace the runtime observer and does not infer live gray
//! failures from source code alone.

use crate::heuristics::{HeuristicEntry, StaticPrior, StaticPriorSet, DEFAULT_ENTRIES};
use crate::CRATE_VERSION;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const MAX_EVIDENCE_PER_HEURISTIC: usize = 6;
const MAX_EVIDENCE_PER_SIGNAL: usize = 4;
const DSSE_PAYLOAD_TYPE: &str = "application/vnd.in-toto+json";
const DSFB_PREDICATE_TYPE: &str =
    "https://github.com/infinityabundance/dsfb-gray/attestations/crate-scan/v1";
pub(crate) const AUDIT_SCORE_METHOD: &str = "dsfb-assurance-score-v1";
/// Default root directory under which timestamped scan runs are written.
pub const DEFAULT_SCAN_OUTPUT_ROOT: &str = "output-dsfb-gray";

/// One line of source evidence supporting a heuristic motif match.
#[derive(Debug, Clone)]
pub struct ScanEvidence {
    /// File containing the matched pattern.
    pub path: PathBuf,
    /// 1-based line number of the match.
    pub line_number: usize,
    /// Pattern that triggered the evidence hit.
    pub pattern: &'static str,
    /// Trimmed source snippet from the matching line.
    pub snippet: String,
}

/// Aggregated source-motif match for one heuristic entry.
#[derive(Debug, Clone)]
pub struct HeuristicSourceMatch {
    /// The matched heuristic entry.
    pub heuristic: HeuristicEntry,
    /// Distinct patterns that matched.
    pub matched_patterns: Vec<&'static str>,
    /// Example evidence lines for the match.
    pub evidence: Vec<ScanEvidence>,
    /// Total line-level hits across all scanned files.
    pub total_hits: usize,
}

/// Result of scanning a source tree for DSFB heuristic provenance motifs.
#[derive(Debug, Clone)]
pub struct CrateSourceScanReport {
    /// Legacy interpretation hint retained for compatibility.
    ///
    /// DSFB now emits one canonical broad audit. This field no longer changes
    /// the evidence set, score denominator, or primary report structure.
    pub profile: ScanProfile,
    /// Crate name if found in Cargo.toml, else the directory name.
    pub crate_name: String,
    /// Crate version if found in Cargo.toml.
    pub crate_version: Option<String>,
    /// UTC timestamp at which the scan report was generated.
    pub generated_at_utc: String,
    /// Root directory that was scanned.
    pub root: PathBuf,
    /// Deterministic SHA-256 digest of the scanned crate tree.
    pub source_sha256: String,
    /// VCS commit hint from `.cargo_vcs_info.json` if present.
    pub vcs_commit: Option<String>,
    /// Path inside the source VCS if present.
    pub path_in_vcs: Option<String>,
    /// Number of source files scanned.
    pub files_scanned: usize,
    /// Matched heuristic motifs sorted by hit count.
    pub matched_heuristics: Vec<HeuristicSourceMatch>,
    /// Caveat describing what this scan means.
    pub caveat: &'static str,
    certification: CertificationProfile,
}

/// Legacy interpretation hint for the DSFB static crate scan.
///
/// DSFB now emits one canonical broad audit and uses conclusion lenses at the
/// end of the report instead of primary profile-driven report identities. This
/// enum is retained so older callers can still pass an interpretation hint
/// while the scanner continues to collect the same full evidence set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanProfile {
    /// General-purpose legacy hint.
    General,
    /// Legacy cloud-native / async-service hint.
    CloudNative,
    /// Legacy distributed systems / consensus / networking hint.
    DistributedSystems,
    /// Legacy industrial / safety-monitoring hint.
    IndustrialSafety,
    /// Legacy supply-chain / provenance / secure-build hint.
    SupplyChain,
}

impl ScanProfile {
    /// Parse a CLI/profile string.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "general" => Some(Self::General),
            "cloud" | "cloud-native" | "cloud_native" => Some(Self::CloudNative),
            "distributed" | "distributed-systems" | "distributed_systems" => {
                Some(Self::DistributedSystems)
            }
            "industrial" | "industrial-safety" | "industrial_safety" | "safety" => {
                Some(Self::IndustrialSafety)
            }
            "supply-chain" | "supply_chain" | "supplychain" => Some(Self::SupplyChain),
            _other => None,
        }
    }

    /// Stable string identifier used by the CLI and machine-readable outputs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::CloudNative => "cloud-native",
            Self::DistributedSystems => "distributed-systems",
            Self::IndustrialSafety => "industrial-safety",
            Self::SupplyChain => "supply-chain",
        }
    }

    /// Human-readable title for this profile.
    pub fn title(self) -> &'static str {
        match self {
            Self::General => "General Rust Crate Review",
            Self::CloudNative => "Cloud-Native Service Review",
            Self::DistributedSystems => "Distributed Systems Review",
            Self::IndustrialSafety => "Industrial / Safety Review",
            Self::SupplyChain => "Supply-Chain / Provenance Review",
        }
    }

    /// Explanatory note describing what this profile emphasizes.
    pub fn focus(self) -> &'static str {
        match self {
            Self::General => {
                "Balanced interpretation across safety, verification, lifecycle, and structural findings."
            }
            Self::CloudNative => {
                "Emphasizes async behavior, backpressure, cancellation, detached tasks, and operational noise under service load."
            }
            Self::DistributedSystems => {
                "Emphasizes heartbeat timing, clock integrity, queue growth, retry behavior, quorum-sensitive logic, and partial-write/networking hazards."
            }
            Self::IndustrialSafety => {
                "Emphasizes bounded behavior, fail-safe state handling, resource determinism, Power-of-Ten proxies, and reviewability."
            }
            Self::SupplyChain => {
                "Emphasizes provenance, dependency drift, dynamic loading, FFI surface, lifecycle artifacts, and attestation portability."
            }
        }
    }
}

/// Ed25519 signing material for DSSE export.
#[derive(Debug, Clone)]
pub struct ScanSigningKey {
    key_id: String,
    signing_key: SigningKey,
}

impl ScanSigningKey {
    /// Load a signing key from the environment.
    ///
    /// The secret may be provided as 32-byte hex or base64 in
    /// `DSFB_SCAN_SIGNING_KEY`. `DSFB_SCAN_KEY_ID` overrides the derived key id.
    pub fn from_environment() -> io::Result<Option<Self>> {
        let Some(secret) = env::var_os("DSFB_SCAN_SIGNING_KEY") else {
            return Ok(None);
        };
        let secret = secret.into_string().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "signing key must be UTF-8")
        })?;
        let key_id = env::var("DSFB_SCAN_KEY_ID")
            .ok()
            .filter(|value| !value.trim().is_empty());
        Self::from_secret_text(&secret, key_id.as_deref()).map(Some)
    }

    /// Parse a signing key from 32-byte hex or base64 text.
    pub fn from_secret_text(secret: &str, key_id: Option<&str>) -> io::Result<Self> {
        let secret_bytes = parse_secret_key(secret)?;
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let derived_key_id = match key_id {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _fallback => {
                let public_key = signing_key.verifying_key().to_bytes();
                format!("ed25519:{}", hex_encode(&public_key[..8]))
            }
        };

        Ok(Self {
            key_id: derived_key_id,
            signing_key,
        })
    }

    fn key_id(&self) -> &str {
        &self.key_id
    }

    fn sign(&self, payload_type: &str, payload: &[u8]) -> String {
        let pae = dsse_pae(payload_type, payload);
        let signature = self.signing_key.sign(&pae);
        BASE64_STANDARD.encode(signature.to_bytes())
    }
}

/// Paths of JSON scan artifacts written to disk.
#[derive(Debug, Clone)]
pub struct ScanArtifactPaths {
    /// Timestamped run directory containing all scan outputs.
    pub output_dir: PathBuf,
    /// Human-readable text report export.
    pub report_path: PathBuf,
    /// SARIF findings export.
    pub sarif_path: PathBuf,
    /// in-toto statement export.
    pub statement_path: PathBuf,
    /// DSSE envelope export.
    pub dsse_path: PathBuf,
    /// Whether the DSSE envelope contains a signature.
    pub signed: bool,
}

/// Paths describing one timestamped scan-output run.
#[derive(Debug, Clone)]
pub struct ScanRunPaths {
    /// Base output root under which the run directory was created.
    pub base_output_root: PathBuf,
    /// Timestamped output directory for this scan invocation.
    pub run_dir: PathBuf,
    /// UTC timestamp string used in the run directory name.
    pub timestamp_utc: String,
}

#[derive(Debug, Clone, Default)]
struct VcsInfo {
    git_commit: Option<String>,
    path_in_vcs: Option<String>,
}

#[derive(Debug, Clone)]
struct CertificationProfile {
    runtime: RuntimeProfile,
    safety: SafetyProfile,
    verification: VerificationProfile,
    build: BuildProfile,
    lifecycle: LifecycleProfile,
    power_of_ten: PowerOfTenProfile,
    advanced: AdvancedStructuralProfile,
    audit_score: AuditScoreCard,
    manifest: ManifestMetadata,
    artifacts_inspected: usize,
}

#[derive(Debug, Clone)]
struct AuditScoreCard {
    overall_percent: f64,
    earned_weighted_points: f64,
    possible_weighted_points: f64,
    band: &'static str,
    sections: Vec<AuditScoreSection>,
}

#[derive(Debug, Clone)]
struct AuditScoreSection {
    id: &'static str,
    title: &'static str,
    weight_percent: f64,
    checkpoint_count: usize,
    earned_checkpoints: f64,
    section_percent: f64,
    weighted_points: f64,
}

#[derive(Debug, Clone)]
struct PowerOfTenProfile {
    rules: Vec<PowerOfTenRuleAudit>,
}

#[derive(Debug, Clone)]
struct AdvancedStructuralProfile {
    checks: Vec<AdvancedStructuralCheck>,
    hotspots: Vec<CriticalityHotspot>,
}

#[derive(Debug, Clone)]
struct PowerOfTenRuleAudit {
    number: u8,
    title: &'static str,
    status: PowerOfTenStatus,
    detail: String,
    evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PowerOfTenStatus {
    Applied,
    NotApplied,
    Indeterminate,
}

#[derive(Debug, Clone)]
struct AdvancedStructuralCheck {
    id: &'static str,
    title: &'static str,
    status: StructuralCheckStatus,
    detail: String,
    evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructuralCheckStatus {
    Elevated,
    Clear,
    Indeterminate,
}

#[derive(Debug, Clone)]
struct CanonicalFinding {
    id: String,
    title: String,
    category: &'static str,
    status_label: &'static str,
    severity_rank: usize,
    classification: &'static str,
    confidence: &'static str,
    impact_kind: &'static str,
    rust_why: &'static str,
    readiness_why: &'static str,
    detail: String,
    remediation: &'static str,
    verification: &'static str,
    evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone)]
struct AdvisorySubscore {
    id: &'static str,
    title: &'static str,
    percent: f64,
    basis: &'static str,
}

#[derive(Debug, Clone)]
struct CriticalityHotspot {
    path: PathBuf,
    function_name: String,
    start_line: usize,
    estimated_complexity: usize,
    risk_score: usize,
    signals: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct RuntimeProfile {
    no_std_declared: bool,
    no_std_evidence: Vec<ScanEvidence>,
    alloc_crate_hits: usize,
    alloc_evidence: Vec<ScanEvidence>,
    heap_allocation_hits: usize,
    heap_allocation_evidence: Vec<ScanEvidence>,
    runtime_core_alloc_hits: usize,
    runtime_core_heap_allocation_hits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsafeCodePolicy {
    Forbid,
    Deny,
    NotDeclared,
}

#[derive(Debug, Clone)]
struct SafetyProfile {
    unsafe_policy: UnsafeCodePolicy,
    unsafe_policy_evidence: Vec<ScanEvidence>,
    unsafe_sites: usize,
    unsafe_evidence: Vec<ScanEvidence>,
    panic_sites: usize,
    panic_evidence: Vec<ScanEvidence>,
    unwrap_sites: usize,
    unwrap_evidence: Vec<ScanEvidence>,
    ffi_sites: usize,
    ffi_evidence: Vec<ScanEvidence>,
    safety_comment_sites: usize,
    safety_comment_evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone)]
struct VerificationProfile {
    tests_dir_present: bool,
    test_marker_hits: usize,
    test_marker_evidence: Vec<ScanEvidence>,
    property_testing_hits: usize,
    property_testing_evidence: Vec<ScanEvidence>,
    concurrency_exploration_hits: usize,
    concurrency_exploration_evidence: Vec<ScanEvidence>,
    fuzzing_hits: usize,
    fuzzing_evidence: Vec<ScanEvidence>,
    formal_methods_hits: usize,
    formal_methods_evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone)]
struct BuildProfile {
    direct_dependencies: usize,
    build_dependencies: usize,
    dev_dependencies: usize,
    has_build_script: bool,
    proc_macro_crate: bool,
    codegen_hits: usize,
    codegen_evidence: Vec<ScanEvidence>,
}

#[derive(Debug, Clone)]
struct LifecycleProfile {
    readme_present: bool,
    changelog_present: bool,
    security_md_present: bool,
    safety_md_present: bool,
    architecture_doc_present: bool,
    docs_dir_present: bool,
    license_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct ManifestMetadata {
    crate_name: Option<String>,
    crate_version: Option<String>,
    edition: Option<String>,
    license: Option<String>,
    rust_version: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
    documentation: Option<String>,
    readme: Option<String>,
    build_script: Option<String>,
    proc_macro: bool,
    direct_dependencies: usize,
    build_dependencies: usize,
    dev_dependencies: usize,
}

#[derive(Debug, Clone)]
struct SourceDocument {
    relative_path: PathBuf,
    contents: String,
    analysis_contents: String,
    risk_contents: String,
}

#[derive(Debug, Clone)]
struct FunctionSummary {
    path: PathBuf,
    name: String,
    lowered_name: String,
    lowered_signature: String,
    lowered_attributes: String,
    start_line: usize,
    line_count: usize,
    body: String,
    lowered_body: String,
    assertion_count: usize,
    estimated_complexity: usize,
}

#[derive(Debug, Clone)]
struct PatternScan {
    total_hits: usize,
    matched_patterns: Vec<&'static str>,
    evidence: Vec<ScanEvidence>,
}

struct PatternSpec {
    heuristic_id: &'static str,
    patterns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManifestSection {
    None,
    Package,
    Lib,
    Dependencies,
    BuildDependencies,
    DevDependencies,
}

const PATTERN_SPECS: &[PatternSpec] = &[
    PatternSpec {
        heuristic_id: "H-ALLOC-01",
        patterns: &[
            "vec::with_capacity",
            ".reserve(",
            " reserve(",
            "reserve_exact(",
        ],
    },
    PatternSpec {
        heuristic_id: "H-LOCK-01",
        patterns: &[
            "rwlock",
            "tokio::sync::rwlock",
            "std::sync::rwlock",
            "parking_lot::rwlock",
        ],
    },
    PatternSpec {
        heuristic_id: "H-RAFT-01",
        patterns: &["openraft", "election_timeout", "heartbeat", "leader lease"],
    },
    PatternSpec {
        heuristic_id: "H-ASYNC-01",
        patterns: &[
            "spawn_blocking",
            "block_in_place",
            "thread::sleep(",
            "std::thread::sleep",
        ],
    },
    PatternSpec {
        heuristic_id: "H-TCP-01",
        patterns: &["tcpstream", "tcplistener", "socket", "connect(", "accept("],
    },
    PatternSpec {
        heuristic_id: "H-CHAN-01",
        patterns: &[
            "sync::mpsc",
            "mpsc::channel(",
            "mpsc::unbounded_channel",
            "bounded channel",
        ],
    },
    PatternSpec {
        heuristic_id: "H-CLOCK-01",
        patterns: &[
            "instant::now()",
            "systemtime::now()",
            "monotonic",
            "timestamp",
        ],
    },
    PatternSpec {
        heuristic_id: "H-THRU-01",
        patterns: &["throughput", "ops/sec", "bytes/sec", "qps"],
    },
    PatternSpec {
        heuristic_id: "H-SERDE-01",
        patterns: &[
            "serde",
            "serialize",
            "deserialize",
            "serde_json",
            "bincode",
            "prost",
        ],
    },
    PatternSpec {
        heuristic_id: "H-GRPC-01",
        patterns: &[
            "tonic::",
            "\"tonic\"",
            "flow control",
            "window_size",
            "http/2",
            "http2",
        ],
    },
    PatternSpec {
        heuristic_id: "H-DNS-01",
        patterns: &["dns", "resolver", "trust-dns", "hickory"],
    },
    PatternSpec {
        heuristic_id: "H-ERR-01",
        patterns: &[
            "timeout",
            "pool exhaustion",
            "retry",
            "backoff",
            "trysenderror",
        ],
    },
];

const NO_STD_PATTERNS: &[&str] = &[
    "#![no_std]",
    "cfg_attr(not(feature = \"std\"), no_std)",
    "cfg_attr(not(any(feature = \"std\")), no_std)",
    " no_std)]",
];

const ALLOC_PATTERNS: &[&str] = &[
    "extern crate alloc",
    "use alloc::",
    "alloc::vec::vec",
    "alloc::string::string",
    "alloc::boxed::box",
];

const HEAP_PATTERNS: &[&str] = &[
    "vec::new(",
    "vec::with_capacity(",
    "string::new(",
    "string::with_capacity(",
    "box::new(",
    "arc::new(",
    "rc::new(",
    "hashmap<",
    "hashset<",
    "btreemap<",
    "btreeset<",
    "vecdeque<",
    "binaryheap<",
    "format!(",
    ".to_string()",
    ".to_owned()",
];

const FORBID_UNSAFE_PATTERNS: &[&str] = &[
    "#![forbid(unsafe_code)]",
    "#![cfg_attr(not(test), forbid(unsafe_code))]",
];
const DENY_UNSAFE_PATTERNS: &[&str] = &[
    "#![deny(unsafe_code)]",
    "#![cfg_attr(not(test), deny(unsafe_code))]",
];
const UNSAFE_PATTERNS: &[&str] = &[
    "unsafe {",
    "unsafe{",
    "unsafe fn",
    "unsafe impl",
    "unsafe trait",
    "unsafe extern",
];
const PANIC_PATTERNS: &[&str] = &[
    "panic!(",
    "todo!(",
    "unimplemented!(",
    "unreachable!(",
    "panic_any(",
];
const UNWRAP_PATTERNS: &[&str] = &[".unwrap(", ".expect(", ".unwrap_err(", ".expect_err("];
const FFI_PATTERNS: &[&str] = &[
    "extern \"c\"",
    "#[repr(c)]",
    "#[no_mangle]",
    "cxx::bridge",
    "bindgen::",
    "[dependencies.bindgen]",
    "[build-dependencies.bindgen]",
    ".dependencies.bindgen]",
    ".build-dependencies.bindgen]",
    "::ffi",
];
const SAFETY_COMMENT_PATTERNS: &[&str] = &["safety:"];

const TEST_PATTERNS: &[&str] = &["#[test]", "#[tokio::test]", "#[cfg(test)]", "mod tests"];
const PROPERTY_TEST_PATTERNS: &[&str] = &["proptest!", "quickcheck", "bolero", "arbtest"];
const CONCURRENCY_EXPLORATION_PATTERNS: &[&str] = &["loom::", "shuttle::"];
const FUZZING_PATTERNS: &[&str] = &[
    "libfuzzer_sys",
    "cargo fuzz",
    "honggfuzz",
    "afl::",
    "arbitrary::",
];
const FORMAL_METHOD_PATTERNS: &[&str] = &["kani", "creusot", "prusti", "flux::"];

const CODEGEN_PATTERNS: &[&str] = &[
    "bindgen::",
    "[dependencies.bindgen]",
    "[build-dependencies.bindgen]",
    ".dependencies.bindgen]",
    ".build-dependencies.bindgen]",
    "cc::build",
    "cmake::config",
    "cxx_build",
    "prost_build",
    "tonic_build",
    "lalrpop",
    "autocfg",
    "vergen::",
    "include!(concat!(env!(\"out_dir\"",
];

const INTERIOR_MUTABILITY_PATTERNS: &[&str] = &[
    "cell<",
    "cell::",
    "refcell<",
    "refcell::",
    "unsafecell<",
    "unsafecell::",
    "atomicbool",
    "atomicu",
    "atomici",
    "atomicptr",
];

const ASYNC_LOCK_PATTERNS: &[&str] = &[
    ".lock().await",
    ".read().await",
    ".write().await",
    "tokio::sync::mutex",
    "tokio::sync::rwlock",
    "mutexguard",
    "rwlockreadguard",
    "rwlockwriteguard",
];

const CATCH_ALL_MATCH_PATTERNS: &[&str] = &["_ =>"];

const HARD_CODED_WAIT_PATTERNS: &[&str] = &[
    "duration::from_millis(",
    "duration::from_secs(",
    "tokio::time::sleep(",
    "std::thread::sleep(",
    "thread::sleep(",
    "sleep_until(",
];

const DYNAMIC_LOADING_PATTERNS: &[&str] =
    &["libloading", "dlopen", "loadlibrary", "getprocaddress"];

const RESOURCE_LIFECYCLE_PATTERNS: &[&str] = &[
    "mem::forget(",
    "manuallydrop<",
    "into_raw_fd(",
    "from_raw_fd(",
    "into_raw_handle(",
    "from_raw_handle(",
    "memmap",
    "mmap",
];

const COMMAND_BUFFER_PATTERNS: &[&str] = &[
    "mpsc::channel",
    "mpsc::unbounded_channel",
    "tokio::sync::mpsc",
    "crossbeam_channel",
    "crossbeam::channel",
];

const TTL_GUARD_PATTERNS: &[&str] = &[
    "ttl",
    "deadline",
    "expires",
    "stale",
    "sequence",
    "nonce",
    "generation",
];

const INTERRUPT_ATTRIBUTE_PATTERNS: &[&str] =
    &["#[interrupt]", "#[interrupt(", "#[cortex_m_rt::interrupt]"];

const ISR_FORBIDDEN_PATTERNS: &[&str] = &[
    "vec::new(",
    "vec::with_capacity(",
    "string::new(",
    "string::with_capacity(",
    "box::new(",
    ".lock(",
    ".lock().await",
    ".read().await",
    ".write().await",
    "std::sync::mutex",
    "tokio::sync::mutex",
    "parking_lot::mutex",
];

const ITERATOR_TERMINAL_PATTERNS: &[&str] = &[
    ".collect(",
    ".collect::<",
    ".fold(",
    ".count(",
    ".last(",
    ".sum(",
];
const ITERATOR_BOUND_PATTERNS: &[&str] = &[".take(", ".nth(", ".next()"];
const OPEN_ENDED_ITERATOR_PATTERNS: &[&str] = &[
    "impl iterator",
    "iterator<",
    "read_dir(",
    "args_os(",
    "args(",
    "receiver",
    "stream",
];

const MANUAL_POLL_PENDING_PATTERNS: &[&str] = &["poll::pending", "return poll::pending"];
const WAKE_PATTERNS: &[&str] = &["wake_by_ref(", ".wake()", "cx.waker()", "context.waker()"];

const JOIN_HANDLE_DISCARD_SPAWN_PATTERNS: &[&str] = &[
    "tokio::spawn(",
    "tokio::task::spawn(",
    "tokio::spawn_blocking(",
    "tokio::task::spawn_blocking(",
];
const JOIN_HANDLE_DISCARD_CONTEXT_PATTERNS: &[&str] =
    &["let _ =", "_ =", "drop(", "std::mem::drop("];

const RELAXED_ORDERING_PATTERNS: &[&str] = &["ordering::relaxed"];
const CRITICAL_STATE_PATTERNS: &[&str] = &[
    "quorum",
    "leader",
    "election",
    "lease",
    "term",
    "epoch",
    "heartbeat",
    "commit",
    "state",
];

const WRITE_CALL_PATTERNS: &[&str] = &[".write("];
const WRITE_HANDLING_PATTERNS: &[&str] = &[
    "write_all(",
    "errorkind::interrupted",
    "wouldblock",
    "shortwrite",
];

const ASYNC_RECURSION_PATTERNS: &[&str] = &["#[async_recursion", "async_recursion]"];
const DEPTH_BOUND_PATTERNS: &[&str] = &["depth", "limit", "max_depth", "remaining"];

const UNBOUNDED_CHANNEL_PATTERNS: &[&str] = &["mpsc::unbounded_channel"];

const READ_BUFFER_SIGNATURE_PATTERNS: &[&str] = &[
    "&[u8]",
    "&mut [u8]",
    "bytes",
    "bytesmut",
    "packet",
    "frame",
    "buffer",
];
const COPY_ON_READ_PATTERNS: &[&str] = &[".to_vec()", ".clone()"];

const ASSERT_PATTERNS: &[&str] = &[
    "assert!(",
    "assert_eq!(",
    "assert_ne!(",
    "debug_assert!(",
    "debug_assert_eq!(",
    "debug_assert_ne!(",
];

const P10_RULE1_PATTERNS: &[&str] = &["goto ", "setjmp", "longjmp", "#[async_recursion"];
const P10_RULE7_EXPLICIT_IGNORE_PATTERNS: &[&str] = &["let _ =", ".ok();", ".err();"];
const P10_RULE8_MACRO_PATTERNS: &[&str] = &[
    "macro_rules!",
    "#[proc_macro]",
    "#[proc_macro_derive]",
    "#[proc_macro_attribute]",
];
const P10_RULE10_WARNING_PATTERNS: &[&str] = &[
    "-d warnings",
    "#![deny(warnings)]",
    "deny(warnings)",
    "warnings = \"deny\"",
];
const P10_RULE10_ANALYZER_PATTERNS: &[&str] = &[
    "cargo clippy",
    "clippy::",
    "cargo audit",
    "cargo deny",
    "miri",
    "kani",
    "prusti",
    "creusot",
];

/// Scan a crate source tree and return a static motif report.
pub fn scan_crate_source(root: &Path) -> io::Result<CrateSourceScanReport> {
    scan_crate_source_with_profile(root, ScanProfile::General)
}

/// Scan a crate source tree using a legacy interpretation hint.
///
/// DSFB now emits one canonical broad audit. This compatibility entrypoint
/// retains the older call shape while continuing to collect the same full
/// evidence set.
pub fn scan_crate_source_with_profile(
    root: &Path,
    profile: ScanProfile,
) -> io::Result<CrateSourceScanReport> {
    let root = root.canonicalize()?;
    let all_files = collect_files(&root)?;
    let generated_at_utc = generated_scan_timestamp();
    let source_sha256 = compute_tree_sha256(&root, &all_files)?;
    let vcs_info = scan_vcs_info(&root);
    let artifact_documents = load_documents(&root, &all_files);
    let source_files = collect_source_scan_files(&all_files);
    let documents = load_documents(&root, &source_files);
    let manifest = scan_manifest(&root.join("Cargo.toml"));
    let (crate_name, crate_version) = crate_identity_from_manifest(&root, &manifest);
    let matched_heuristics = scan_matched_heuristics(&documents);
    let certification = build_certification_profile(
        &root,
        &all_files,
        &documents,
        &artifact_documents,
        &manifest,
    );

    Ok(build_crate_scan_report(ScanReportInputs {
        profile,
        generated_at_utc,
        root,
        source_sha256,
        vcs_info,
        files_scanned: source_files.len(),
        crate_name,
        crate_version,
        matched_heuristics,
        certification,
    }))
}

fn generated_scan_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn collect_source_scan_files(all_files: &[PathBuf]) -> Vec<PathBuf> {
    all_files
        .iter()
        .filter(|path| is_source_scan_file(path))
        .cloned()
        .collect()
}

fn crate_identity_from_manifest(
    root: &Path,
    manifest: &ManifestMetadata,
) -> (String, Option<String>) {
    let crate_name = manifest.crate_name.clone().unwrap_or_else(|| {
        root.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown-crate")
            .to_string()
    });
    (crate_name, manifest.crate_version.clone())
}

fn scan_matched_heuristics(documents: &[SourceDocument]) -> Vec<HeuristicSourceMatch> {
    let mut matched_heuristics = Vec::new();

    for entry in DEFAULT_ENTRIES {
        let Some(spec) = PATTERN_SPECS
            .iter()
            .find(|spec| spec.heuristic_id == entry.id.0)
        else {
            continue;
        };

        let scan = scan_patterns(documents, spec.patterns, MAX_EVIDENCE_PER_HEURISTIC);
        if scan.total_hits > 0 {
            matched_heuristics.push(HeuristicSourceMatch {
                heuristic: *entry,
                matched_patterns: scan.matched_patterns,
                evidence: scan.evidence,
                total_hits: scan.total_hits,
            });
        }
    }

    matched_heuristics.sort_by(|a, b| {
        b.total_hits
            .cmp(&a.total_hits)
            .then_with(|| a.heuristic.id.0.cmp(b.heuristic.id.0))
    });
    matched_heuristics
}

fn build_certification_profile(
    root: &Path,
    all_files: &[PathBuf],
    documents: &[SourceDocument],
    artifact_documents: &[SourceDocument],
    manifest: &ManifestMetadata,
) -> CertificationProfile {
    let runtime = scan_runtime_profile(documents);
    let safety = scan_safety_profile(documents);
    let verification = scan_verification_profile(all_files, documents);
    let build = scan_build_profile(root, documents, manifest);
    let lifecycle = scan_lifecycle_profile(all_files);
    let functions = extract_function_summaries(documents);
    let power_of_ten = scan_power_of_ten_profile(
        documents,
        artifact_documents,
        &functions,
        &runtime,
        &safety,
        &build,
    );
    let advanced =
        scan_advanced_structural_profile(documents, artifact_documents, &functions, &safety);
    let audit_score = build_audit_scorecard(
        &safety,
        &verification,
        &build,
        &lifecycle,
        manifest,
        &power_of_ten,
        &advanced,
    );

    CertificationProfile {
        runtime,
        safety,
        verification,
        build,
        lifecycle,
        power_of_ten,
        advanced,
        audit_score,
        manifest: manifest.clone(),
        artifacts_inspected: all_files.len(),
    }
}

struct ScanReportInputs {
    profile: ScanProfile,
    generated_at_utc: String,
    root: PathBuf,
    source_sha256: String,
    vcs_info: VcsInfo,
    files_scanned: usize,
    crate_name: String,
    crate_version: Option<String>,
    matched_heuristics: Vec<HeuristicSourceMatch>,
    certification: CertificationProfile,
}

fn build_crate_scan_report(inputs: ScanReportInputs) -> CrateSourceScanReport {
    CrateSourceScanReport {
        profile: inputs.profile,
        crate_name: inputs.crate_name,
        crate_version: inputs.crate_version,
        generated_at_utc: inputs.generated_at_utc,
        root: inputs.root,
        source_sha256: inputs.source_sha256,
        vcs_commit: inputs.vcs_info.git_commit,
        path_in_vcs: inputs.vcs_info.path_in_vcs,
        files_scanned: inputs.files_scanned,
        matched_heuristics: inputs.matched_heuristics,
        caveat: "Static source-visible proxy only: this report highlights structural motifs, constrained-runtime signals, verification evidence, and lifecycle artifacts. It does not certify the crate or infer live gray failures without runtime telemetry.",
        certification: inputs.certification,
    }
}

/// Derive bounded runtime structural priors from a static crate scan.
///
/// These priors are intentionally conservative. They bias the runtime observer
/// toward motifs that are visibly present in the source tree, but they do not
/// override runtime evidence or guarantee that a live failure exists.
pub fn derive_static_priors_from_scan(report: &CrateSourceScanReport) -> StaticPriorSet {
    report
        .matched_heuristics
        .iter()
        .fold(StaticPriorSet::new(), |priors, matched| {
            let confidence = ((matched.total_hits as f64).ln_1p() / 4.0).clamp(0.15, 0.95);
            let drift_scale = (1.0 - 0.25 * confidence).clamp(0.75, 1.0);
            let slew_scale = if matched.heuristic.slew_threshold > 0.0 {
                (1.0 - 0.30 * confidence).clamp(0.70, 1.0)
            } else {
                1.0
            };
            priors.with_prior(StaticPrior::new(
                matched.heuristic.id,
                confidence,
                drift_scale,
                slew_scale,
            ))
        })
}

/// Render a source scan report as plain text.
pub fn render_scan_report(report: &CrateSourceScanReport) -> String {
    let findings = collect_canonical_findings(report);
    let advisory_subscores = advisory_subscores(report);
    let derived_priors = derive_static_priors_from_scan(report);
    let mut out = String::with_capacity(8192);
    render_scan_report_header(&mut out, report);

    render_audit_summary(&mut out, report, &findings);
    render_report_badge_section(&mut out, report);
    render_audit_score_section(&mut out, report, &advisory_subscores);
    render_top_findings(&mut out, &findings);
    render_hotspots_section(&mut out, &report.certification.advanced.hotspots);
    render_code_quality_themes(&mut out, &findings);
    render_remediation_guide(&mut out, &findings);
    render_verification_suggestions(&mut out, &findings);
    render_evidence_ledger(&mut out, &findings);
    render_detailed_audit_surface(&mut out, report);
    render_derived_priors_section(&mut out, report, &derived_priors);
    render_heuristic_motif_section(&mut out, report);
    render_conclusion_lenses(&mut out, report, &findings);
    out
}

fn render_scan_report_header(out: &mut String, report: &CrateSourceScanReport) {
    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str("║         DSFB Gray Static Crate Scan Report                 ║\n");
    out.push_str("║   Canonical Broad Audit for Code Quality + Review Readiness║\n");
    out.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

    out.push_str(&format!("Crate: {}\n", report.crate_name));
    if let Some(version) = &report.crate_version {
        out.push_str(&format!("Version: {}\n", version));
    }
    out.push_str(&format!(
        "Generated At (UTC): {}\n",
        report.generated_at_utc
    ));
    out.push_str(&format!("Root: {}\n", report.root.display()));
    out.push_str(&format!(
        "Scanned Crate: https://crates.io/crates/{}\n",
        report.crate_name
    ));
    out.push_str(&format!(
        "Scanned Crate Docs: https://docs.rs/{}\n",
        report.crate_name
    ));
    out.push_str("Scanner Crate: https://crates.io/crates/dsfb-gray\n");
    out.push_str("Scanner Docs: https://docs.rs/dsfb-gray\n");
    out.push_str(&format!("Source SHA-256: {}\n", report.source_sha256));
    out.push_str(&format!(
        "VCS Commit: {}\n",
        report.vcs_commit.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "Path In VCS: {}\n",
        report.path_in_vcs.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!("Source Files Scanned: {}\n", report.files_scanned));
    out.push_str(&format!(
        "Artifact Files Inspected: {}\n",
        report.certification.artifacts_inspected
    ));
    out.push_str(&format!(
        "Matched Heuristics: {}\n",
        report.matched_heuristics.len()
    ));
    out.push_str(&format!("Caveat: {}\n\n", report.caveat));
}

fn render_detailed_audit_surface(out: &mut String, report: &CrateSourceScanReport) {
    out.push_str("Detailed Audit Surface\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(
        "The sections below preserve the full DSFB audit breadth. They are detailed evidence views, not separate scan modes.\n\n",
    );
    render_runtime_section(out, &report.certification.runtime);
    render_safety_section(out, &report.certification.safety);
    render_verification_section(out, &report.certification.verification);
    render_build_section(out, &report.certification.build);
    render_lifecycle_section(
        out,
        &report.certification.lifecycle,
        &report.certification.manifest,
    );
    render_power_of_ten_section(out, &report.certification.power_of_ten);
    render_advanced_structural_section(out, &report.certification.advanced);
}

fn render_derived_priors_section(
    out: &mut String,
    report: &CrateSourceScanReport,
    derived_priors: &StaticPriorSet,
) {
    out.push_str("Derived Runtime Structural Priors\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(
        "These bounded priors are derived from static source motifs. They are meant to bias runtime review toward structurally plausible motifs, not to override runtime evidence.\n",
    );
    if report.matched_heuristics.is_empty() {
        out.push_str("No static priors derived because no DSFB source motifs matched.\n\n");
        return;
    }

    for matched in &report.matched_heuristics {
        if let Some(prior) = derived_priors.get(matched.heuristic.id) {
            out.push_str(&format!(
                "{} confidence={:.2} drift_scale={:.2} slew_scale={:.2}\n",
                matched.heuristic.id.0, prior.confidence, prior.drift_scale, prior.slew_scale,
            ));
        }
    }
    out.push('\n');
}

fn render_heuristic_motif_section(out: &mut String, report: &CrateSourceScanReport) {
    out.push_str("DSFB Heuristic Motifs\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if report.matched_heuristics.is_empty() {
        out.push_str("No DSFB source motifs matched.\n\n");
        return;
    }

    for matched in &report.matched_heuristics {
        render_heuristic_motif(out, matched);
    }
}

fn render_heuristic_motif(out: &mut String, matched: &HeuristicSourceMatch) {
    out.push_str(&format!(
        "{} → {:?}\n",
        matched.heuristic.id.0, matched.heuristic.reason_code
    ));
    out.push_str(&format!(
        "  Description: {}\n",
        matched.heuristic.description
    ));
    out.push_str(&format!(
        "  Provenance:  {}\n",
        matched.heuristic.provenance
    ));
    out.push_str(&format!("  Total Hits:  {}\n", matched.total_hits));
    out.push_str(&format!(
        "  Patterns:    {}\n",
        matched.matched_patterns.join(", ")
    ));
    out.push_str(&format!(
        "  Remediation: {}\n",
        heuristic_remediation(matched.heuristic.id.0)
    ));
    render_named_evidence_block(out, "  Evidence", matched.heuristic.id.0, &matched.evidence);
    out.push_str(&format!(
        "  Classification: {}\n",
        heuristic_classification(matched.heuristic.id.0)
    ));
    out.push_str(&format!(
        "  Confidence: {}\n",
        heuristic_confidence(matched.total_hits)
    ));
    out.push_str(&format!(
        "  Impact Kind: {}\n",
        heuristic_impact_kind(matched.heuristic.id.0)
    ));
    out.push_str(&format!(
        "  Why This Matters In Rust: {}\n",
        heuristic_rust_why(matched.heuristic.id.0)
    ));
    out.push_str(&format!(
        "  Review / Readiness Note: {}\n",
        heuristic_readiness_why(matched.heuristic.id.0)
    ));
    out.push_str(&format!(
        "  Verification Suggestion: {}\n",
        heuristic_verification_suggestion(matched.heuristic.id.0)
    ));
    out.push('\n');
}

/// Render the scan as SARIF 2.1.0 JSON.
pub fn render_scan_sarif(report: &CrateSourceScanReport) -> String {
    serde_json::to_string_pretty(&build_sarif_value(report))
        .unwrap_or_else(|err| format!("{{\"error\":\"failed to render SARIF: {err}\"}}"))
}

/// Render the scan as an in-toto statement whose predicate summarizes the scan.
pub fn render_scan_attestation_statement(report: &CrateSourceScanReport) -> String {
    let sarif_json = render_scan_sarif(report);
    serde_json::to_string_pretty(&build_attestation_statement_value(report, &sarif_json))
        .unwrap_or_else(|err| {
            format!("{{\"error\":\"failed to render in-toto statement: {err}\"}}")
        })
}

/// Render the scan as a DSSE envelope around the in-toto statement.
///
/// When `signer` is `None`, the envelope is emitted without signatures.
pub fn render_scan_dsse_envelope(
    report: &CrateSourceScanReport,
    signer: Option<&ScanSigningKey>,
) -> String {
    let statement_json = render_scan_attestation_statement(report);
    let payload = statement_json.as_bytes();
    let signatures = signer
        .map(|signer| {
            vec![json!({
                "keyid": signer.key_id(),
                "sig": signer.sign(DSSE_PAYLOAD_TYPE, payload),
            })]
        })
        .unwrap_or_default();

    serde_json::to_string_pretty(&json!({
        "payloadType": DSSE_PAYLOAD_TYPE,
        "payload": BASE64_STANDARD.encode(payload),
        "signatures": signatures,
    }))
    .unwrap_or_else(|err| format!("{{\"error\":\"failed to render DSSE envelope: {err}\"}}"))
}

/// Create a timestamped output directory for one scan invocation.
pub fn prepare_scan_output_run(base_output_root: &Path) -> io::Result<ScanRunPaths> {
    let timestamp_utc = scan_run_timestamp(OffsetDateTime::now_utc());
    prepare_scan_output_run_at(base_output_root, &timestamp_utc)
}

/// Move legacy root-level scan artifacts into one timestamped migration folder.
pub fn migrate_legacy_scan_artifacts(
    legacy_root: &Path,
    base_output_root: &Path,
) -> io::Result<Option<PathBuf>> {
    let legacy_files = collect_legacy_scan_artifacts(legacy_root)?;
    if legacy_files.is_empty() {
        return Ok(None);
    }

    fs::create_dir_all(base_output_root)?;
    let migration_dir = create_unique_run_dir(
        base_output_root,
        &format!(
            "dsfb-gray-{}-migration",
            scan_run_timestamp(OffsetDateTime::now_utc())
        ),
    )?;

    for legacy_path in legacy_files {
        let Some(file_name) = legacy_path.file_name() else {
            continue;
        };
        fs::rename(&legacy_path, migration_dir.join(file_name))?;
    }

    Ok(Some(migration_dir))
}

/// Write SARIF, in-toto, and DSSE artifacts for a completed scan.
pub fn export_scan_artifacts(
    report: &CrateSourceScanReport,
    out_dir: &Path,
    signer: Option<&ScanSigningKey>,
) -> io::Result<ScanArtifactPaths> {
    fs::create_dir_all(out_dir)?;
    let stem = scan_artifact_stem(report);
    let report_path = out_dir.join(format!("{stem}.txt"));
    let sarif_path = out_dir.join(format!("{stem}.sarif.json"));
    let statement_path = out_dir.join(format!("{stem}.intoto.json"));
    let dsse_path = out_dir.join(format!("{stem}.dsse.json"));

    fs::write(&report_path, render_scan_report(report))?;
    fs::write(&sarif_path, render_scan_sarif(report))?;
    fs::write(&statement_path, render_scan_attestation_statement(report))?;
    fs::write(&dsse_path, render_scan_dsse_envelope(report, signer))?;

    Ok(ScanArtifactPaths {
        output_dir: out_dir.to_path_buf(),
        report_path,
        sarif_path,
        statement_path,
        dsse_path,
        signed: signer.is_some(),
    })
}

fn build_sarif_value(report: &CrateSourceScanReport) -> Value {
    let findings = collect_canonical_findings(report);
    let advisory_subscores = advisory_subscores(report);
    let (rules, results) = build_sarif_rules_and_results(report);

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "DSFB Gray Scanner",
                    "version": CRATE_VERSION,
                    "informationUri": "https://github.com/infinityabundance/dsfb-gray",
                    "rules": rules,
                }
            },
            "automationDetails": {
                "id": "dsfb-gray/crate-scan",
            },
            "invocations": [{
                "executionSuccessful": true,
                "endTimeUtc": report.generated_at_utc,
            }],
            "results": results,
            "properties": build_sarif_properties(report, &advisory_subscores, &findings),
        }]
    })
}

fn build_attestation_statement_value(report: &CrateSourceScanReport, sarif_json: &str) -> Value {
    let sarif_sha256 = sha256_hex(sarif_json.as_bytes());
    let structural_priors = derive_static_priors_from_scan(report);
    let findings = collect_canonical_findings(report);
    let advisory_subscores = advisory_subscores(report);

    json!({
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [build_attestation_subject(report)],
        "predicateType": DSFB_PREDICATE_TYPE,
        "predicate": build_attestation_predicate(
            report,
            sarif_sha256,
            &structural_priors,
            &findings,
            &advisory_subscores,
        ),
    })
}

fn build_sarif_rules_and_results(report: &CrateSourceScanReport) -> (Vec<Value>, Vec<Value>) {
    let mut rules = Vec::new();
    let mut results = Vec::new();
    append_heuristic_sarif_entries(&mut rules, &mut results, report);
    append_power_of_ten_sarif_entries(&mut rules, &mut results, report);
    append_advanced_sarif_entries(&mut rules, &mut results, report);
    (rules, results)
}

fn append_heuristic_sarif_entries(
    rules: &mut Vec<Value>,
    results: &mut Vec<Value>,
    report: &CrateSourceScanReport,
) {
    for matched in &report.matched_heuristics {
        rules.push(sarif_rule_for_heuristic(matched));
        results.push(sarif_result_for_heuristic(matched));
    }
}

fn sarif_rule_for_heuristic(matched: &HeuristicSourceMatch) -> Value {
    json!({
        "id": matched.heuristic.id.0,
        "name": matched.heuristic.id.0,
        "shortDescription": { "text": matched.heuristic.description },
        "fullDescription": { "text": matched.heuristic.provenance },
        "help": { "text": heuristic_remediation(matched.heuristic.id.0) },
        "properties": {
            "dsfbCategory": "heuristic",
            "reasonCode": format!("{:?}", matched.heuristic.reason_code),
            "classification": heuristic_classification(matched.heuristic.id.0),
            "confidence": heuristic_confidence(matched.total_hits),
            "impactKind": heuristic_impact_kind(matched.heuristic.id.0),
            "guidanceOnly": true,
        }
    })
}

fn sarif_result_for_heuristic(matched: &HeuristicSourceMatch) -> Value {
    json!({
        "ruleId": matched.heuristic.id.0,
        "level": "warning",
        "kind": "review",
        "message": {
            "text": format!(
                "{} matched {} source motif hit(s) with reason code {:?}.",
                matched.heuristic.id.0,
                matched.total_hits,
                matched.heuristic.reason_code
            )
        },
        "locations": sarif_locations(&matched.evidence),
        "properties": {
            "dsfbCategory": "heuristic",
            "totalHits": matched.total_hits,
            "matchedPatterns": matched.matched_patterns,
            "provenance": matched.heuristic.provenance,
            "classification": heuristic_classification(matched.heuristic.id.0),
            "confidence": heuristic_confidence(matched.total_hits),
            "impactKind": heuristic_impact_kind(matched.heuristic.id.0),
            "verificationSuggestion": heuristic_verification_suggestion(matched.heuristic.id.0),
            "remediation": heuristic_remediation(matched.heuristic.id.0),
            "evidenceIds": evidence_ids(matched.heuristic.id.0, &matched.evidence),
        }
    })
}

fn append_power_of_ten_sarif_entries(
    rules: &mut Vec<Value>,
    results: &mut Vec<Value>,
    report: &CrateSourceScanReport,
) {
    for rule in &report.certification.power_of_ten.rules {
        if rule.status == PowerOfTenStatus::Applied {
            continue;
        }
        rules.push(sarif_rule_for_power_of_ten(rule));
        results.push(sarif_result_for_power_of_ten(rule));
    }
}

fn sarif_rule_for_power_of_ten(rule: &PowerOfTenRuleAudit) -> Value {
    json!({
        "id": format!("P10-{}", rule.number),
        "name": format!("P10-{}", rule.number),
        "shortDescription": { "text": rule.title },
        "fullDescription": { "text": rule.detail },
        "help": { "text": power_of_ten_remediation(rule.number) },
        "properties": {
            "dsfbCategory": "nasa-power-of-ten",
            "status": power_of_ten_status_label(rule.status),
            "classification": power_of_ten_classification(rule.number),
            "confidence": power_of_ten_confidence(rule.status, rule.evidence.len()),
            "impactKind": power_of_ten_impact_kind(rule.number),
            "guidanceOnly": true,
        }
    })
}

fn sarif_result_for_power_of_ten(rule: &PowerOfTenRuleAudit) -> Value {
    let rule_id = format!("P10-{}", rule.number);
    json!({
        "ruleId": rule_id,
        "level": if rule.status == PowerOfTenStatus::NotApplied { "warning" } else { "note" },
        "kind": "review",
        "message": { "text": format!("{}: {}", rule.title, rule.detail) },
        "locations": sarif_locations(&rule.evidence),
        "properties": {
            "dsfbCategory": "nasa-power-of-ten",
            "status": power_of_ten_status_label(rule.status),
            "classification": power_of_ten_classification(rule.number),
            "confidence": power_of_ten_confidence(rule.status, rule.evidence.len()),
            "impactKind": power_of_ten_impact_kind(rule.number),
            "verificationSuggestion": power_of_ten_verification_suggestion(rule.number),
            "remediation": power_of_ten_remediation(rule.number),
            "evidenceIds": evidence_ids(&rule_id, &rule.evidence),
        }
    })
}

fn append_advanced_sarif_entries(
    rules: &mut Vec<Value>,
    results: &mut Vec<Value>,
    report: &CrateSourceScanReport,
) {
    for check in &report.certification.advanced.checks {
        if check.status == StructuralCheckStatus::Clear {
            continue;
        }
        rules.push(sarif_rule_for_advanced_check(check));
        results.push(sarif_result_for_advanced_check(check));
    }
}

fn sarif_rule_for_advanced_check(check: &AdvancedStructuralCheck) -> Value {
    json!({
        "id": check.id,
        "name": check.id,
        "shortDescription": { "text": check.title },
        "fullDescription": { "text": check.detail },
        "help": { "text": advanced_check_remediation(check.id) },
        "properties": {
            "dsfbCategory": "advanced-structural",
            "status": structural_check_status_label(check.status),
            "classification": advanced_check_classification(check.id),
            "confidence": advanced_check_confidence(check.status, check.evidence.len()),
            "impactKind": advanced_check_impact_kind(check.id),
            "guidanceOnly": true,
        }
    })
}

fn sarif_result_for_advanced_check(check: &AdvancedStructuralCheck) -> Value {
    json!({
        "ruleId": check.id,
        "level": if check.status == StructuralCheckStatus::Elevated { "warning" } else { "note" },
        "kind": "review",
        "message": { "text": format!("{}: {}", check.title, check.detail) },
        "locations": sarif_locations(&check.evidence),
        "properties": {
            "dsfbCategory": "advanced-structural",
            "status": structural_check_status_label(check.status),
            "classification": advanced_check_classification(check.id),
            "confidence": advanced_check_confidence(check.status, check.evidence.len()),
            "impactKind": advanced_check_impact_kind(check.id),
            "verificationSuggestion": advanced_check_verification_suggestion(check.id),
            "remediation": advanced_check_remediation(check.id),
            "evidenceIds": evidence_ids(check.id, &check.evidence),
        }
    })
}

fn build_sarif_properties(
    report: &CrateSourceScanReport,
    advisory_subscores: &[AdvisorySubscore],
    findings: &[CanonicalFinding],
) -> Value {
    json!({
        "crateName": report.crate_name,
        "crateVersion": report.crate_version,
        "auditMode": "canonical-broad-audit",
        "sourceRoot": report.root.display().to_string(),
        "sourceSha256": report.source_sha256,
        "vcsCommit": report.vcs_commit,
        "pathInVcs": report.path_in_vcs,
        "filesScanned": report.files_scanned,
        "artifactsInspected": report.certification.artifacts_inspected,
        "auditScore": audit_score_json(&report.certification.audit_score),
        "advisorySubscores": advisory_subscores_json(advisory_subscores),
        "guidanceSemantics": {
            "codeQualityGoal": true,
            "reviewReadinessGoal": true,
            "nonCertificationStatement": "DSFB does not certify compliance with IEC, ISO, RTCA, MIL, NIST, or other standards. Use this audit as a guideline for improvement and review readiness."
        },
        "conclusionLenses": conclusion_lenses_json(report, findings),
    })
}

fn advisory_subscores_json(advisory_subscores: &[AdvisorySubscore]) -> Vec<Value> {
    advisory_subscores
        .iter()
        .map(|subscore| {
            json!({
                "id": subscore.id,
                "title": subscore.title,
                "percent": round_percent(subscore.percent),
                "basis": subscore.basis,
            })
        })
        .collect()
}

fn build_attestation_subject(report: &CrateSourceScanReport) -> Value {
    let subject_name = match &report.crate_version {
        Some(version) => format!("pkg:cargo/{}@{}", report.crate_name, version),
        None => format!("pkg:cargo/{}", report.crate_name),
    };
    json!({
        "name": subject_name,
        "digest": { "sha256": report.source_sha256 }
    })
}

fn build_attestation_predicate(
    report: &CrateSourceScanReport,
    sarif_sha256: String,
    structural_priors: &StaticPriorSet,
    findings: &[CanonicalFinding],
    advisory_subscores: &[AdvisorySubscore],
) -> Value {
    json!({
        "generatedAtUtc": report.generated_at_utc,
        "scanner": attestation_scanner_json(),
        "guidanceSemantics": {
            "codeQualityGoal": true,
            "reviewReadinessGoal": true,
            "nonCertificationStatement": "DSFB findings may support internal review against standards-oriented expectations, but DSFB does not certify compliance with IEC, ISO, RTCA, MIL, NIST, or other standards.",
        },
        "crate": attestation_crate_json(report),
        "sarif": {
            "mediaType": "application/sarif+json",
            "sha256": sarif_sha256,
            "resultCount": build_sarif_result_count(report),
        },
        "summary": build_attestation_summary(report, structural_priors, findings, advisory_subscores),
    })
}

fn attestation_scanner_json() -> Value {
    json!({
        "name": "dsfb-gray",
        "version": CRATE_VERSION,
        "recipeVersion": 1,
        "auditMode": "canonical-broad-audit",
        "recipe": [
            "heuristics-bank-default",
            "constrained-runtime-audit",
            "unsafe-panic-ffi-audit",
            "verification-audit",
            "lifecycle-audit",
            "nasa-power-of-ten-audit",
            "advanced-structural-audit"
        ]
    })
}

fn attestation_crate_json(report: &CrateSourceScanReport) -> Value {
    json!({
        "name": report.crate_name,
        "version": report.crate_version,
        "root": report.root.display().to_string(),
        "sourceSha256": report.source_sha256,
        "vcsCommit": report.vcs_commit,
        "pathInVcs": report.path_in_vcs,
        "filesScanned": report.files_scanned,
        "artifactsInspected": report.certification.artifacts_inspected,
    })
}

fn build_attestation_summary(
    report: &CrateSourceScanReport,
    structural_priors: &StaticPriorSet,
    findings: &[CanonicalFinding],
    advisory_subscores: &[AdvisorySubscore],
) -> Value {
    let (power_applied, power_not_applied, power_indeterminate) =
        power_of_ten_status_counts(report);
    let (advanced_elevated, advanced_clear, advanced_indeterminate) =
        advanced_status_counts(report);
    json!({
        "auditScore": audit_score_json(&report.certification.audit_score),
        "advisorySubscores": advisory_subscores_json(advisory_subscores),
        "matchedHeuristics": attestation_heuristics_json(report, structural_priors),
        "powerOfTen": {
            "applied": power_applied,
            "notApplied": power_not_applied,
            "indeterminate": power_indeterminate,
        },
        "advancedStructural": {
            "elevated": advanced_elevated,
            "clear": advanced_clear,
            "indeterminate": advanced_indeterminate,
        },
        "criticalityHotspots": attestation_hotspots_json(report),
        "findings": attestation_findings_json(findings),
        "conclusionLenses": conclusion_lenses_json(report, findings),
    })
}

fn power_of_ten_status_counts(report: &CrateSourceScanReport) -> (usize, usize, usize) {
    let applied = report
        .certification
        .power_of_ten
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::Applied)
        .count();
    let not_applied = report
        .certification
        .power_of_ten
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::NotApplied)
        .count();
    let indeterminate = report
        .certification
        .power_of_ten
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::Indeterminate)
        .count();
    (applied, not_applied, indeterminate)
}

fn advanced_status_counts(report: &CrateSourceScanReport) -> (usize, usize, usize) {
    let elevated = report
        .certification
        .advanced
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Elevated)
        .count();
    let clear = report
        .certification
        .advanced
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Clear)
        .count();
    let indeterminate = report
        .certification
        .advanced
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Indeterminate)
        .count();
    (elevated, clear, indeterminate)
}

fn attestation_heuristics_json(
    report: &CrateSourceScanReport,
    structural_priors: &StaticPriorSet,
) -> Vec<Value> {
    report
        .matched_heuristics
        .iter()
        .map(|matched| {
            let prior = structural_priors.get(matched.heuristic.id);
            json!({
                "id": matched.heuristic.id.0,
                "reasonCode": format!("{:?}", matched.heuristic.reason_code),
                "totalHits": matched.total_hits,
                "matchedPatterns": matched.matched_patterns,
                "structuralPrior": prior.map(|prior| json!({
                    "confidence": prior.confidence,
                    "driftScale": prior.drift_scale,
                    "slewScale": prior.slew_scale,
                })),
            })
        })
        .collect()
}

fn attestation_hotspots_json(report: &CrateSourceScanReport) -> Vec<Value> {
    report
        .certification
        .advanced
        .hotspots
        .iter()
        .map(|hotspot| {
            json!({
                "path": hotspot.path.display().to_string(),
                "line": hotspot.start_line,
                "function": hotspot.function_name,
                "riskScore": hotspot.risk_score,
                "estimatedComplexity": hotspot.estimated_complexity,
                "signals": hotspot.signals,
            })
        })
        .collect()
}

fn attestation_findings_json(findings: &[CanonicalFinding]) -> Vec<Value> {
    findings
        .iter()
        .map(|finding| {
            json!({
                "id": finding.id,
                "title": finding.title,
                "category": finding.category,
                "status": finding.status_label,
                "classification": finding.classification,
                "confidence": finding.confidence,
                "impactKind": finding.impact_kind,
                "remediation": finding.remediation,
                "verificationSuggestion": finding.verification,
                "evidenceIds": evidence_ids(&finding.id, &finding.evidence),
            })
        })
        .collect()
}

fn build_sarif_result_count(report: &CrateSourceScanReport) -> usize {
    let p10_results = report
        .certification
        .power_of_ten
        .rules
        .iter()
        .filter(|rule| rule.status != PowerOfTenStatus::Applied)
        .count();
    let advanced_results = report
        .certification
        .advanced
        .checks
        .iter()
        .filter(|check| check.status != StructuralCheckStatus::Clear)
        .count();

    report.matched_heuristics.len() + p10_results + advanced_results
}

fn collect_canonical_findings(report: &CrateSourceScanReport) -> Vec<CanonicalFinding> {
    let mut findings = collect_heuristic_findings(report);
    append_power_of_ten_findings(&mut findings, report);
    append_advanced_findings(&mut findings, report);
    sort_canonical_findings(&mut findings);
    findings
}

fn collect_heuristic_findings(report: &CrateSourceScanReport) -> Vec<CanonicalFinding> {
    report
        .matched_heuristics
        .iter()
        .map(build_heuristic_finding)
        .collect()
}

fn build_heuristic_finding(matched: &HeuristicSourceMatch) -> CanonicalFinding {
    CanonicalFinding {
        id: matched.heuristic.id.0.to_string(),
        title: matched.heuristic.description.to_string(),
        category: "heuristic",
        status_label: "matched",
        severity_rank: 40 + matched.total_hits.min(20),
        classification: heuristic_classification(matched.heuristic.id.0),
        confidence: heuristic_confidence(matched.total_hits),
        impact_kind: heuristic_impact_kind(matched.heuristic.id.0),
        rust_why: heuristic_rust_why(matched.heuristic.id.0),
        readiness_why: heuristic_readiness_why(matched.heuristic.id.0),
        detail: matched.heuristic.provenance.to_string(),
        remediation: heuristic_remediation(matched.heuristic.id.0),
        verification: heuristic_verification_suggestion(matched.heuristic.id.0),
        evidence: matched.evidence.clone(),
    }
}

fn append_power_of_ten_findings(
    findings: &mut Vec<CanonicalFinding>,
    report: &CrateSourceScanReport,
) {
    findings.extend(
        report
            .certification
            .power_of_ten
            .rules
            .iter()
            .filter(|rule| rule.status != PowerOfTenStatus::Applied)
            .map(build_power_of_ten_finding),
    );
}

fn build_power_of_ten_finding(rule: &PowerOfTenRuleAudit) -> CanonicalFinding {
    CanonicalFinding {
        id: format!("P10-{}", rule.number),
        title: rule.title.to_string(),
        category: "nasa-power-of-ten",
        status_label: power_of_ten_status_label(rule.status),
        severity_rank: match rule.status {
            PowerOfTenStatus::NotApplied => 85,
            PowerOfTenStatus::Indeterminate => 60,
            PowerOfTenStatus::Applied => 0,
        },
        classification: power_of_ten_classification(rule.number),
        confidence: power_of_ten_confidence(rule.status, rule.evidence.len()),
        impact_kind: power_of_ten_impact_kind(rule.number),
        rust_why: power_of_ten_rust_why(rule.number),
        readiness_why: power_of_ten_readiness_why(rule.number),
        detail: rule.detail.clone(),
        remediation: power_of_ten_remediation(rule.number),
        verification: power_of_ten_verification_suggestion(rule.number),
        evidence: rule.evidence.clone(),
    }
}

fn append_advanced_findings(findings: &mut Vec<CanonicalFinding>, report: &CrateSourceScanReport) {
    findings.extend(
        report
            .certification
            .advanced
            .checks
            .iter()
            .filter(|check| check.status != StructuralCheckStatus::Clear)
            .map(build_advanced_finding),
    );
}

fn build_advanced_finding(check: &AdvancedStructuralCheck) -> CanonicalFinding {
    CanonicalFinding {
        id: check.id.to_string(),
        title: check.title.to_string(),
        category: "advanced-structural",
        status_label: structural_check_status_label(check.status),
        severity_rank: match check.status {
            StructuralCheckStatus::Elevated => 90,
            StructuralCheckStatus::Indeterminate => 65,
            StructuralCheckStatus::Clear => 0,
        },
        classification: advanced_check_classification(check.id),
        confidence: advanced_check_confidence(check.status, check.evidence.len()),
        impact_kind: advanced_check_impact_kind(check.id),
        rust_why: advanced_check_rust_why(check.id),
        readiness_why: advanced_check_readiness_why(check.id),
        detail: check.detail.clone(),
        remediation: advanced_check_remediation(check.id),
        verification: advanced_check_verification_suggestion(check.id),
        evidence: check.evidence.clone(),
    }
}

fn sort_canonical_findings(findings: &mut [CanonicalFinding]) {
    findings.sort_by(|a, b| {
        b.severity_rank
            .cmp(&a.severity_rank)
            .then_with(|| b.evidence.len().cmp(&a.evidence.len()))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn advisory_subscores(report: &CrateSourceScanReport) -> Vec<AdvisorySubscore> {
    let safety = score_section_percent(&report.certification.audit_score, "safety");
    let verification = score_section_percent(&report.certification.audit_score, "verification");
    let build = score_section_percent(&report.certification.audit_score, "build");
    let lifecycle = score_section_percent(&report.certification.audit_score, "lifecycle");
    let power = score_section_percent(&report.certification.audit_score, "nasa_power_of_ten");
    let advanced = score_section_percent(&report.certification.audit_score, "advanced_structural");

    vec![
        advisory_correctness_subscore(report, safety),
        advisory_maintainability_subscore(report, lifecycle),
        advisory_concurrency_subscore(report),
        advisory_resource_subscore(report),
        advisory_verification_subscore(report, verification, build),
        advisory_assurance_subscore(safety, verification, build, lifecycle, power, advanced),
    ]
}

fn advisory_correctness_subscore(report: &CrateSourceScanReport, safety: f64) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "correctness",
        title: "Correctness",
        percent: mean_percent(&[
            safety,
            selected_power_of_ten_percent(&report.certification.power_of_ten, &[5, 7, 9]),
            selected_advanced_percent(
                &report.certification.advanced,
                &[
                    "SAFE-STATE",
                    "FUTURE-WAKE",
                    "DROP-PANIC",
                    "CLOCK-MIX",
                    "SHORT-WRITE",
                    "ATOMIC-RELAXED",
                ],
            ),
        ]),
        basis:
            "Derived from safety surface, correctness-critical Power-of-Ten rules, and correctness-oriented structural checks.",
    }
}

fn advisory_maintainability_subscore(
    report: &CrateSourceScanReport,
    lifecycle: f64,
) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "maintainability",
        title: "Maintainability",
        percent: mean_percent(&[
            lifecycle,
            selected_power_of_ten_percent(&report.certification.power_of_ten, &[4, 6, 8]),
            selected_advanced_percent(
                &report.certification.advanced,
                &["ITER-UNB", "CARGO-VERS", "PART-SPACE"],
            ),
        ]),
        basis:
            "Derived from lifecycle/governance evidence, reviewability-oriented Power-of-Ten rules, and maintainability-heavy structural checks.",
    }
}

fn advisory_concurrency_subscore(report: &CrateSourceScanReport) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "concurrency_async",
        title: "Concurrency / Async",
        percent: mean_percent(&[
            selected_advanced_percent(
                &report.certification.advanced,
                &[
                    "ASYNC-LOCK",
                    "FUTURE-WAKE",
                    "TASK-LEAK",
                    "ASYNC-RECUR",
                    "CHAN-UNB",
                ],
            ),
            selected_power_of_ten_percent(&report.certification.power_of_ten, &[2, 5]),
        ]),
        basis:
            "Derived from async/concurrency structural checks and bounded-control-flow review signals.",
    }
}

fn advisory_resource_subscore(report: &CrateSourceScanReport) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "resource_discipline",
        title: "Resource Discipline",
        percent: mean_percent(&[
            runtime_resource_percent(&report.certification.runtime),
            selected_advanced_percent(
                &report.certification.advanced,
                &[
                    "ALLOC-HOT",
                    "CWE-404",
                    "CMD-BUF",
                    "ITER-UNB",
                    "ZERO-COPY",
                    "SHORT-WRITE",
                ],
            ),
            selected_power_of_ten_percent(&report.certification.power_of_ten, &[2, 3]),
        ]),
        basis:
            "Derived from runtime-allocation proxies, resource-lifecycle checks, and bounded-allocation / bounded-loop review rules.",
    }
}

fn advisory_verification_subscore(
    report: &CrateSourceScanReport,
    verification: f64,
    build: f64,
) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "verification_reviewability",
        title: "Verification / Reviewability",
        percent: mean_percent(&[
            verification,
            build,
            selected_power_of_ten_percent(&report.certification.power_of_ten, &[4, 8, 10]),
        ]),
        basis:
            "Derived from verification signals, build/tooling complexity, and analyzability-oriented Power-of-Ten rules.",
    }
}

fn advisory_assurance_subscore(
    safety: f64,
    verification: f64,
    build: f64,
    lifecycle: f64,
    power: f64,
    advanced: f64,
) -> AdvisorySubscore {
    AdvisorySubscore {
        id: "assurance_provenance",
        title: "Assurance / Provenance",
        percent: mean_percent(&[safety, verification, build, lifecycle, power, advanced]),
        basis:
            "Derived from the full locked rubric as a broad readiness-oriented advisory synthesis.",
    }
}

fn score_section_percent(score: &AuditScoreCard, id: &str) -> f64 {
    score
        .sections
        .iter()
        .find(|section| section.id == id)
        .map(|section| section.section_percent)
        .unwrap_or(0.0)
}

fn selected_power_of_ten_percent(profile: &PowerOfTenProfile, rules: &[u8]) -> f64 {
    let values = profile
        .rules
        .iter()
        .filter(|rule| rules.contains(&rule.number))
        .map(|rule| match rule.status {
            PowerOfTenStatus::Applied => 100.0,
            PowerOfTenStatus::Indeterminate => 50.0,
            PowerOfTenStatus::NotApplied => 0.0,
        })
        .collect::<Vec<_>>();
    mean_percent(&values)
}

fn selected_advanced_percent(profile: &AdvancedStructuralProfile, ids: &[&str]) -> f64 {
    let values = profile
        .checks
        .iter()
        .filter(|check| ids.contains(&check.id))
        .map(|check| match check.status {
            StructuralCheckStatus::Clear => 100.0,
            StructuralCheckStatus::Indeterminate => 50.0,
            StructuralCheckStatus::Elevated => 0.0,
        })
        .collect::<Vec<_>>();
    mean_percent(&values)
}

fn runtime_resource_percent(profile: &RuntimeProfile) -> f64 {
    mean_percent(&[
        score_threshold(profile.alloc_crate_hits, 0, 2) * 100.0,
        score_threshold(profile.heap_allocation_hits, 0, 6) * 100.0,
    ])
}

fn mean_percent(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn heuristic_classification(id: &str) -> &'static str {
    match id {
        "H-ASYNC-01" | "H-CHAN-01" | "H-LOCK-01" | "H-ALLOC-01" | "H-SERDE-01" => "design-review",
        "H-ERR-01" => "defect-candidate",
        _other => "context-needed",
    }
}

fn heuristic_confidence(total_hits: usize) -> &'static str {
    match total_hits {
        0..=1 => "low",
        2..=4 => "medium",
        _other => "high",
    }
}

fn heuristic_impact_kind(id: &str) -> &'static str {
    match id {
        "H-ASYNC-01" | "H-LOCK-01" | "H-CHAN-01" | "H-GRPC-01" => "concurrency/async",
        "H-ALLOC-01" | "H-THRU-01" | "H-SERDE-01" => "resource discipline",
        "H-RAFT-01" | "H-TCP-01" | "H-CLOCK-01" | "H-ERR-01" | "H-DNS-01" => "correctness",
        _other => "reviewability",
    }
}

fn heuristic_rust_why(id: &str) -> &'static str {
    match id {
        "H-ASYNC-01" => "Blocking motifs inside async-heavy code often translate into executor unfairness, starvation, and misleading service-level symptoms.",
        "H-CHAN-01" => "Channel motifs are often where Rust services hide backpressure, queue growth, and detached ownership assumptions.",
        "H-ALLOC-01" => "Allocation-heavy source motifs often correlate with hot-path latency variance and avoidable memory churn.",
        "H-LOCK-01" => "Shared-lock motifs concentrate contention and can turn otherwise-local latency into system-wide tail behavior.",
        "H-CLOCK-01" => "Clock-related motifs are where monotonic and wall-clock assumptions can quietly diverge.",
        "H-ERR-01" => "Retry and timeout motifs are where error handling frequently shifts from resilience into amplification.",
        _other => "The matched motif is source-visible and reviewable in Rust code, but it still needs local reasoning before it should drive design changes.",
    }
}

fn heuristic_readiness_why(id: &str) -> &'static str {
    match id {
        "H-ASYNC-01" | "H-CHAN-01" | "H-ERR-01" | "H-CLOCK-01" => {
            "This motif frequently appears in assurance-oriented reviews because it changes how operators and reviewers reason about timing, boundedness, and fault handling."
        }
        _other => {
            "This motif can support internal review against standards-oriented expectations, but it is still only a structural proxy rather than compliance evidence by itself."
        }
    }
}

fn heuristic_verification_suggestion(id: &str) -> &'static str {
    match id {
        "H-ASYNC-01" => "Replay a representative async workload and confirm the path yields or offloads before poll duration spikes appear.",
        "H-CHAN-01" => "Exercise a producer-faster-than-consumer test and confirm queue depth remains bounded and observable.",
        "H-ALLOC-01" => "Benchmark the flagged path under steady load and inspect allocation counts before and after preallocation changes.",
        "H-LOCK-01" => "Measure lock hold time or add tracing around the shared path to confirm contention boundaries are explicit.",
        "H-CLOCK-01" => "Add a regression test that isolates monotonic timing logic from wall-clock presentation or protocol boundaries.",
        "H-ERR-01" => "Run a failure-path test and confirm retry pacing, cancellation, and escalation remain bounded.",
        _other => "Review the emitted evidence and add a targeted regression or replay check on the affected path.",
    }
}

fn power_of_ten_classification(rule_number: u8) -> &'static str {
    match rule_number {
        5 | 7 | 9 => "defect-candidate",
        8 | 10 => "review-readiness",
        _other => "design-review",
    }
}

fn power_of_ten_confidence(status: PowerOfTenStatus, evidence_len: usize) -> &'static str {
    match status {
        PowerOfTenStatus::Applied => "high",
        PowerOfTenStatus::NotApplied if evidence_len >= 2 => "high",
        PowerOfTenStatus::NotApplied => "medium",
        PowerOfTenStatus::Indeterminate => "medium",
    }
}

fn power_of_ten_impact_kind(rule_number: u8) -> &'static str {
    match rule_number {
        1 | 2 | 3 | 7 | 9 => "correctness",
        4 | 6 => "maintainability",
        5 => "concurrency/async",
        8 | 10 => "verification/reviewability",
        _other => "assurance/provenance",
    }
}

fn power_of_ten_rust_why(rule_number: u8) -> &'static str {
    match rule_number {
        1 => "Unbounded recursion still threatens reviewability and stack reasoning in Rust, even when ownership is otherwise strong.",
        2 => "Explicit bounds are one of the clearest ways to make Rust control flow auditable under failure pressure.",
        3 => "Steady-state allocation surfaces are often where long-lived Rust services accumulate jitter and memory debt.",
        4 => "Large functions make invariants harder to see, test, and review, even in otherwise safe Rust.",
        5 => "Catch-all state handling often hides missing invariants or incomplete transitions in otherwise exhaustive-looking Rust code.",
        6 => "Global shared state spreads coupling and makes local reasoning harder across modules and tasks.",
        7 => "Unchecked extraction pushes invariant proof onto the reader instead of the code.",
        8 => "Macros and cfg forks can hide large semantic deltas behind small source surfaces.",
        9 => "Raw-pointer and FFI boundaries are where Rust's usual guarantees weaken and local contracts matter most.",
        10 => "Analyzer and warning gates are part of keeping a Rust codebase reviewable over time.",
        _other => "This rule is a Rust-adapted analyzability guideline rather than a language-lawyer restriction.",
    }
}

fn power_of_ten_readiness_why(rule_number: u8) -> &'static str {
    match rule_number {
        8..=10 => {
            "This rule is directly relevant to review readiness because it affects whether a reviewer can trust what paths are present and what tools continue to check."
        }
        _other => {
            "This rule supports bounded, reviewable structure that often matters in compliance- or certification-oriented internal reviews."
        }
    }
}

fn power_of_ten_verification_suggestion(rule_number: u8) -> &'static str {
    match rule_number {
        1 => "Add a focused test or review note that proves the remaining recursion is bounded, or refactor it into an explicit loop/work queue.",
        2 => "Add a regression test that demonstrates a visible loop bound, timeout, or cancellation path on the flagged logic.",
        3 => "Profile the flagged path under steady-state load and confirm no avoidable heap growth remains after initialization.",
        4 => "Split the function and add narrower tests that name the local invariants introduced by the refactor.",
        5 => "Add state-transition tests that cover the previously catch-all path explicitly.",
        6 => "Document ownership/synchronization boundaries or add module-level tests that prove shared state cannot drift silently.",
        7 => "Replace unwrap/expect with explicit handling or add an invariant test that proves the extraction precondition.",
        8 => "Review feature/macro-expanded paths and add CI coverage for the meaningful forks.",
        9 => "Document the local pointer/FFI contract and add the narrowest possible regression around the unsafe edge.",
        10 => "Keep analyzer and warnings-as-errors gates in CI and record the expected toolchain surface in the repo docs.",
        _other => "Review the evidence and add the smallest regression that proves the intended invariant.",
    }
}

fn advanced_check_classification(check_id: &str) -> &'static str {
    match check_id {
        "SAFE-STATE" | "ISR-SAFE" | "FUTURE-WAKE" | "DROP-PANIC" | "CLOCK-MIX" | "SHORT-WRITE"
        | "CWE-404" => "defect-candidate",
        "PLUGIN-LOAD" | "CARGO-VERS" => "review-readiness",
        "ATOMIC-RELAXED" | "CMD-BUF" | "ITER-UNB" => "context-needed",
        _other => "design-review",
    }
}

fn advanced_check_confidence(status: StructuralCheckStatus, evidence_len: usize) -> &'static str {
    match status {
        StructuralCheckStatus::Clear => "high",
        StructuralCheckStatus::Elevated if evidence_len >= 2 => "high",
        StructuralCheckStatus::Elevated => "medium",
        StructuralCheckStatus::Indeterminate => "medium",
    }
}

fn advanced_check_impact_kind(check_id: &str) -> &'static str {
    match check_id {
        "ASYNC-LOCK" | "FUTURE-WAKE" | "TASK-LEAK" | "ASYNC-RECUR" | "CHAN-UNB" => {
            "concurrency/async"
        }
        "ALLOC-HOT" | "CWE-404" | "CMD-BUF" | "ITER-UNB" | "ZERO-COPY" => "resource discipline",
        "SAFE-STATE" | "CLOCK-INTEG" | "CLOCK-MIX" | "SHORT-WRITE" | "DROP-PANIC"
        | "ATOMIC-RELAXED" | "ISR-SAFE" => "correctness",
        "PLUGIN-LOAD" | "CARGO-VERS" | "PART-SPACE" => "assurance/provenance",
        _other => "maintainability",
    }
}

fn advanced_check_rust_why(check_id: &str) -> &'static str {
    match check_id {
        "ASYNC-LOCK" => "Async lock misuse is a common way for otherwise-correct Rust to become operationally brittle under load.",
        "ALLOC-HOT" => "Allocation inside hot loops is often the difference between stable throughput and jitter-heavy Rust services.",
        "CLOCK-MIX" => "Mixing monotonic and wall-clock time is a classic correctness trap in timeout, lease, and control logic.",
        "FUTURE-WAKE" => "Manual futures live on a strict wake contract; getting it wrong produces futures that appear correct but never make progress.",
        "TASK-LEAK" => "Detached tasks are easy to create in Rust async code and hard to reason about during shutdown, overload, or retries.",
        "ZERO-COPY" => "Avoidable copies on hot paths often show up later as bandwidth and tail-latency debt.",
        _other => "This structural check points to a reviewable Rust pattern that often deserves explicit local invariants or tests.",
    }
}

fn advanced_check_readiness_why(check_id: &str) -> &'static str {
    match check_id {
        "PLUGIN-LOAD" | "CARGO-VERS" | "PART-SPACE" => {
            "This finding is especially relevant to review readiness because it affects reproducibility, isolation, or operator trust in what was shipped."
        }
        _other => {
            "This finding can support standards-oriented internal review because it highlights boundedness, determinism, ownership, or resource-discipline questions."
        }
    }
}

fn advanced_check_verification_suggestion(check_id: &str) -> &'static str {
    match check_id {
        "SAFE-STATE" => "Add tests that drive the fallback path explicitly and confirm the intended safe-state behavior is named, not implied.",
        "ASYNC-LOCK" => "Add a focused async regression that proves the lock is dropped before await and that cancellation does not strand shared state.",
        "ALLOC-HOT" => "Benchmark or instrument the flagged loop and confirm allocation count drops after preallocation or refactoring.",
        "CLOCK-INTEG" | "CLOCK-MIX" => "Add tests that isolate monotonic timing behavior from wall-clock use and verify deadline math at the boundary.",
        "RETRY-DAMP" => "Exercise a repeated-failure path and confirm backoff is capped, jittered, and observable.",
        "HARD-WAIT" => "Replace the fixed wait with a state/deadline condition and add a regression that proves the path is now bounded by state, not sleep time.",
        "PART-SPACE" => "Document the shared-resource boundary and add a test or review note that proves ownership/partitioning is intentional.",
        "PLUGIN-LOAD" => "Add review notes or CI checks that prove the dynamic-loading boundary is verified, sandboxed, or intentionally excluded from trusted paths.",
        "CWE-404" => "Exercise an error path and confirm ownership cleanup happens without raw-handle leakage.",
        "CMD-BUF" => "Add queue tests that demonstrate staleness, TTL, cancellation, or sequence handling under backlog.",
        "ITER-UNB" => "Add a bound, trusted finite-source proof, or regression test that demonstrates the iterator cannot grow without limit.",
        "ISR-SAFE" => "Review interrupt-path code and add a targeted test or note proving it stays allocation-free and lock-free where required.",
        "FUTURE-WAKE" => "Add a manual-future regression that proves each Pending path registers a wake before returning.",
        "TASK-LEAK" => "Track JoinHandle ownership and add shutdown tests that prove tasks do not outlive their supervisor unintentionally.",
        "DROP-PANIC" => "Move failure reporting out of Drop and add a regression proving teardown stays infallible under unwind pressure.",
        "ATOMIC-RELAXED" => "Review the state-transition path and add the narrowest concurrency test that proves the ordering is sufficient.",
        "SHORT-WRITE" => "Add IO-path tests that inject Interrupted or partial writes and prove the caller handles them correctly.",
        "ASYNC-RECUR" => "Add a visible depth bound or refactor to a loop/work queue and prove the new path terminates under stress.",
        "CHAN-UNB" => "Add load tests that demonstrate bounded backlog or justify why unbounded growth cannot accumulate invisibly.",
        "ZERO-COPY" => "Benchmark the read path before and after borrowing/reference-counting changes and confirm copy count drops.",
        "CARGO-VERS" => "Pin or narrow version requirements and verify the attested build stays reproducible across fresh environments.",
        _other => "Use the evidence block to write the smallest targeted regression or review note that proves the intended invariant.",
    }
}

fn build_audit_scorecard(
    safety: &SafetyProfile,
    verification: &VerificationProfile,
    build: &BuildProfile,
    lifecycle: &LifecycleProfile,
    manifest: &ManifestMetadata,
    power_of_ten: &PowerOfTenProfile,
    advanced: &AdvancedStructuralProfile,
) -> AuditScoreCard {
    let sections = vec![
        build_safety_score_section(safety),
        build_verification_score_section(verification),
        build_build_score_section(build),
        build_lifecycle_score_section(lifecycle, manifest),
        build_power_of_ten_score_section(power_of_ten),
        build_advanced_score_section(advanced),
    ];
    finalize_audit_scorecard(sections)
}

fn build_safety_score_section(safety: &SafetyProfile) -> AuditScoreSection {
    let checkpoints = [
        score_unsafe_policy(safety.unsafe_policy),
        score_binary(safety.unsafe_sites == 0),
        score_binary(safety.panic_sites == 0),
        score_binary(safety.unwrap_sites == 0),
        safety_ffi_checkpoint(safety),
    ];
    build_score_section("safety", "Safety Surface", 15.0, &checkpoints)
}

fn safety_ffi_checkpoint(safety: &SafetyProfile) -> f64 {
    if safety.ffi_sites == 0 && safety.unsafe_sites == 0 {
        1.0
    } else if safety.safety_comment_sites > 0 {
        0.5
    } else {
        0.0
    }
}

fn build_verification_score_section(verification: &VerificationProfile) -> AuditScoreSection {
    let checkpoints = [
        score_binary(verification.tests_dir_present || verification.test_marker_hits > 0),
        score_binary(verification.property_testing_hits > 0),
        score_binary(verification.concurrency_exploration_hits > 0),
        score_binary(verification.fuzzing_hits > 0),
        score_binary(verification.formal_methods_hits > 0),
    ];
    build_score_section("verification", "Verification Evidence", 15.0, &checkpoints)
}

fn build_build_score_section(build: &BuildProfile) -> AuditScoreSection {
    let checkpoints = [
        score_threshold(build.direct_dependencies, 10, 25),
        score_threshold(build.build_dependencies, 3, 8),
        score_threshold(build.dev_dependencies, 15, 30),
        score_binary(!build.has_build_script),
        score_binary(!build.proc_macro_crate),
        score_binary(build.codegen_hits == 0),
    ];
    build_score_section("build", "Build / Tooling Complexity", 10.0, &checkpoints)
}

fn build_lifecycle_score_section(
    lifecycle: &LifecycleProfile,
    manifest: &ManifestMetadata,
) -> AuditScoreSection {
    let checkpoints = [
        score_binary(lifecycle.readme_present),
        score_binary(lifecycle.changelog_present),
        score_binary(lifecycle.security_md_present),
        score_binary(lifecycle.safety_md_present),
        score_binary(lifecycle.architecture_doc_present),
        score_binary(lifecycle.docs_dir_present),
        score_binary(!lifecycle.license_files.is_empty() || manifest.license.is_some()),
        score_binary(manifest.rust_version.is_some()),
        score_binary(manifest.edition.is_some()),
        score_binary(manifest.repository.is_some()),
        score_binary(manifest.documentation.is_some()),
        score_binary(manifest.homepage.is_some()),
        score_binary(manifest.readme.is_some() || lifecycle.readme_present),
    ];
    build_score_section("lifecycle", "Lifecycle / Governance", 10.0, &checkpoints)
}

fn build_power_of_ten_score_section(power_of_ten: &PowerOfTenProfile) -> AuditScoreSection {
    let checkpoints = power_of_ten
        .rules
        .iter()
        .map(power_of_ten_checkpoint_score)
        .collect::<Vec<_>>();
    build_score_section(
        "nasa_power_of_ten",
        "NASA/JPL Power of Ten",
        25.0,
        &checkpoints,
    )
}

fn power_of_ten_checkpoint_score(rule: &PowerOfTenRuleAudit) -> f64 {
    match rule.status {
        PowerOfTenStatus::Applied => 1.0,
        PowerOfTenStatus::Indeterminate => 0.5,
        PowerOfTenStatus::NotApplied => 0.0,
    }
}

fn build_advanced_score_section(advanced: &AdvancedStructuralProfile) -> AuditScoreSection {
    let checkpoints = advanced
        .checks
        .iter()
        .map(advanced_checkpoint_score)
        .collect::<Vec<_>>();
    build_score_section(
        "advanced_structural",
        "Advanced Structural Checks",
        25.0,
        &checkpoints,
    )
}

fn advanced_checkpoint_score(check: &AdvancedStructuralCheck) -> f64 {
    match check.status {
        StructuralCheckStatus::Clear => 1.0,
        StructuralCheckStatus::Indeterminate => 0.5,
        StructuralCheckStatus::Elevated => 0.0,
    }
}

fn finalize_audit_scorecard(sections: Vec<AuditScoreSection>) -> AuditScoreCard {
    let earned_weighted_points = sections.iter().map(|section| section.weighted_points).sum();
    let possible_weighted_points = sections.iter().map(|section| section.weight_percent).sum();
    let overall_percent = if possible_weighted_points == 0.0 {
        0.0
    } else {
        earned_weighted_points * 100.0 / possible_weighted_points
    };

    AuditScoreCard {
        overall_percent,
        earned_weighted_points,
        possible_weighted_points,
        band: audit_score_band(overall_percent),
        sections,
    }
}

fn build_score_section(
    id: &'static str,
    title: &'static str,
    weight_percent: f64,
    checkpoints: &[f64],
) -> AuditScoreSection {
    let checkpoint_count = checkpoints.len();
    let earned_checkpoints = checkpoints.iter().sum::<f64>();
    let section_ratio = if checkpoint_count == 0 {
        0.0
    } else {
        earned_checkpoints / checkpoint_count as f64
    };
    let section_percent = section_ratio * 100.0;
    let weighted_points = section_ratio * weight_percent;

    AuditScoreSection {
        id,
        title,
        weight_percent,
        checkpoint_count,
        earned_checkpoints,
        section_percent,
        weighted_points,
    }
}

fn score_binary(value: bool) -> f64 {
    if value {
        1.0
    } else {
        0.0
    }
}

fn score_threshold(value: usize, full_threshold: usize, partial_threshold: usize) -> f64 {
    if value <= full_threshold {
        1.0
    } else if value <= partial_threshold {
        0.5
    } else {
        0.0
    }
}

fn score_unsafe_policy(policy: UnsafeCodePolicy) -> f64 {
    match policy {
        UnsafeCodePolicy::Forbid => 1.0,
        UnsafeCodePolicy::Deny => 0.5,
        UnsafeCodePolicy::NotDeclared => 0.0,
    }
}

fn audit_score_band(overall_percent: f64) -> &'static str {
    if overall_percent >= 85.0 {
        "strong assurance posture"
    } else if overall_percent >= 70.0 {
        "developing but substantial assurance posture"
    } else if overall_percent >= 55.0 {
        "mixed assurance posture"
    } else if overall_percent >= 40.0 {
        "limited assurance evidence"
    } else {
        "low assurance readiness"
    }
}

fn audit_score_guideline_lines() -> [&'static str; 5] {
    [
        "Method: weighted checkpoint scoring across Safety (15%), Verification (15%), Build/Tooling (10%), Lifecycle/Governance (10%), NASA/JPL Power of Ten (25%), and Advanced Structural Checks (25%).",
        "Checkpoint credit: pass/clear/applied = 1.0, indeterminate/partial = 0.5, elevated/not applied = 0.0.",
        "Fairness rule: raw motif counts do not linearly reduce the score; each checkpoint contributes once so large crates are not punished simply for having more code.",
        "Informational-only signals such as DSFB heuristic motif matches, hotspot counts, and capability flags like no_std/no_alloc are reported but excluded from the score denominator.",
        "Interpretation: this is a broad improvement and review-readiness score for source-visible controls and evidence, not a certification and not a measure of runtime correctness.",
    ]
}

fn round_percent(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn audit_score_json(score: &AuditScoreCard) -> Value {
    json!({
        "method": AUDIT_SCORE_METHOD,
        "overallPercent": round_percent(score.overall_percent),
        "earnedWeightedPoints": round_percent(score.earned_weighted_points),
        "possibleWeightedPoints": round_percent(score.possible_weighted_points),
        "band": score.band,
        "guideline": audit_score_guideline_lines(),
        "sections": score.sections.iter().map(|section| {
            json!({
                "id": section.id,
                "title": section.title,
                "weightPercent": round_percent(section.weight_percent),
                "checkpointCount": section.checkpoint_count,
                "earnedCheckpoints": round_percent(section.earned_checkpoints),
                "sectionPercent": round_percent(section.section_percent),
                "weightedPoints": round_percent(section.weighted_points),
            })
        }).collect::<Vec<_>>(),
    })
}

fn sarif_locations(evidence: &[ScanEvidence]) -> Vec<Value> {
    evidence
        .iter()
        .map(|item| {
            json!({
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": item.path.display().to_string(),
                    },
                    "region": {
                        "startLine": item.line_number,
                        "snippet": {
                            "text": item.snippet,
                        }
                    }
                }
            })
        })
        .collect()
}

fn render_audit_summary(
    out: &mut String,
    report: &CrateSourceScanReport,
    findings: &[CanonicalFinding],
) {
    let defect_candidates = findings
        .iter()
        .filter(|finding| finding.classification == "defect-candidate")
        .count();
    let design_review = findings
        .iter()
        .filter(|finding| finding.classification == "design-review")
        .count();
    let review_readiness = findings
        .iter()
        .filter(|finding| finding.classification == "review-readiness")
        .count();
    let context_needed = findings
        .iter()
        .filter(|finding| finding.classification == "context-needed")
        .count();

    out.push_str("Audit Summary\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str("Purpose:\n");
    out.push_str("  - improve Rust code quality across the full crate surface\n");
    out.push_str("  - support compliance- and certification-oriented internal review\n");
    out.push_str("  - preserve all current DSFB audit breadth in one canonical report\n");
    out.push_str("Non-certification statement:\n");
    out.push_str("  - DSFB does not certify compliance with IEC, ISO, RTCA, MIL, NIST, or other standards.\n");
    out.push_str(
        "  - Treat this report as a structured guideline for improvement and review readiness.\n",
    );
    out.push_str("Canonical audit shape:\n");
    out.push_str("  - one full audit\n");
    out.push_str("  - one overall score plus visible subscores\n");
    out.push_str("  - one shared evidence set reused by the concluding interpretation lenses\n");
    out.push_str(&format!(
        "Finding mix: {} defect-candidate | {} design-review | {} review-readiness | {} context-needed\n",
        defect_candidates, design_review, review_readiness, context_needed
    ));
    out.push_str(
        "Audit families preserved: runtime, safety, verification, build, lifecycle, Power of Ten, advanced structural, heuristic motifs, runtime priors, and attestation exports.\n\n",
    );
    out.push_str(&format!(
        "Report scope note: DSFB findings may support internal review against standards-oriented expectations, but the report remains a source-visible structural audit of {} artifact(s), not a certificate.\n\n",
        report.certification.artifacts_inspected
    ));
}

fn render_report_badge_section(out: &mut String, report: &CrateSourceScanReport) {
    let report_file_name = format!("{}.txt", scan_artifact_stem(report));
    let badge_snippet = render_report_badge_markdown(report, &report_file_name);
    out.push_str("Add dsfb-gray report badge to your GitHub repo README\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str("DSFB-gray crate: https://crates.io/crates/dsfb-gray\n");
    out.push_str(
        "Use this when you place the audit report in the repository root as a code-quality and review-readiness document.\n",
    );
    out.push_str(&format!(
        "Root-level report link target used below: ./{}\n",
        report_file_name
    ));
    out.push_str("Markdown snippet:\n");
    out.push_str("```md\n");
    out.push_str(&badge_snippet);
    out.push_str("\n```\n");
    out.push_str(
        "Badge semantics: this links to the DSFB audit report for the crate; it is not a compliance or certification badge.\n\n",
    );
}

fn render_audit_score_section(
    out: &mut String,
    report: &CrateSourceScanReport,
    advisory_subscores: &[AdvisorySubscore],
) {
    let score = &report.certification.audit_score;
    out.push_str("Overall Score and Subscores\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!("Scoring Version: {}\n", AUDIT_SCORE_METHOD));
    out.push_str(&format!(
        "Overall: {:.1}% ({})\n",
        score.overall_percent, score.band
    ));
    out.push_str(&format!(
        "Weighted points earned: {:.1}/{:.1}\n",
        score.earned_weighted_points, score.possible_weighted_points
    ));
    out.push_str(
        "Score use: this score is a broad improvement target derived from the locked DSFB audit rubric. It is not a compliance certification.\n",
    );
    render_advisory_subscore_table(out, advisory_subscores);
    render_audit_score_table(out, score);
    out.push_str("Locked rubric section breakdown:\n");
    for section in &score.sections {
        out.push_str(&format!(
            "  - {}: {:.1}% of section, {:.1}/{:.1} weighted points across {} checkpoint(s)\n",
            section.title,
            section.section_percent,
            section.weighted_points,
            section.weight_percent,
            section.checkpoint_count
        ));
    }
    out.push_str("Scoring guideline:\n");
    for line in audit_score_guideline_lines().into_iter() {
        out.push_str(&format!("  - {line}\n"));
    }
    out.push('\n');
}

fn render_report_badge_markdown(report: &CrateSourceScanReport, report_file_name: &str) -> String {
    let score = round_percent(report.certification.audit_score.overall_percent);
    let band = report.certification.audit_score.band;
    let color = badge_color_for_score(score);
    format!(
        "[![DSFB Gray Audit: {:.1}% {}](https://img.shields.io/badge/DSFB%20Gray%20Audit-{:.1}%25-{})](./{})",
        score, band, score, color, report_file_name
    )
}

fn badge_color_for_score(score: f64) -> &'static str {
    if score >= 85.0 {
        "brightgreen"
    } else if score >= 70.0 {
        "green"
    } else if score >= 55.0 {
        "yellowgreen"
    } else if score >= 40.0 {
        "orange"
    } else {
        "red"
    }
}

fn render_top_findings(out: &mut String, findings: &[CanonicalFinding]) {
    out.push_str("Top Findings\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if findings.is_empty() {
        out.push_str("No review-worthy findings were emitted from the current evidence set.\n\n");
        return;
    }

    for finding in findings.iter().take(8) {
        out.push_str(&format!(
            "{} {} [{} | confidence={} | impact={}]\n",
            finding.id,
            finding.status_label,
            finding.classification,
            finding.confidence,
            finding.impact_kind
        ));
        out.push_str(&format!("  Title: {}\n", finding.title));
        out.push_str(&format!("  Detail: {}\n", finding.detail));
        out.push_str(&format!(
            "  Why This Matters In Rust: {}\n",
            finding.rust_why
        ));
        out.push_str(&format!(
            "  Review / Readiness Note: {}\n",
            finding.readiness_why
        ));
        if let Some(first_evidence) = finding.evidence.first() {
            out.push_str(&format!(
                "  First Evidence: {} {}:{} [{}] {}\n",
                evidence_id(&finding.id, first_evidence, 0),
                first_evidence.path.display(),
                first_evidence.line_number,
                first_evidence.pattern,
                first_evidence.snippet
            ));
        }
        out.push('\n');
    }
}

fn render_hotspots_section(out: &mut String, hotspots: &[CriticalityHotspot]) {
    out.push_str("Hotspots\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str("Guide:\n");
    out.push_str("  [##--------] observed  score 12-19\n");
    out.push_str("  [####------] guarded   score 20-29\n");
    out.push_str("  [######----] elevated  score 30-39\n");
    out.push_str("  [########--] high      score 40-49\n");
    out.push_str("  [##########] severe    score 50+\n");
    out.push_str("  row format: path:line `function` [bar] band score=<n> complexity~<n>\n");
    out.push_str("  signals: comma-separated structural risk contributors\n");
    if hotspots.is_empty() {
        out.push_str("No function hotspots extracted.\n\n");
        return;
    }

    for hotspot in hotspots {
        out.push_str(&format!(
            "{}:{} `{}` {} {:<8} score={} complexity~={}\n",
            hotspot.path.display(),
            hotspot.start_line,
            hotspot.function_name,
            heatmap_bar(hotspot.risk_score),
            heatmap_band_label(hotspot.risk_score),
            hotspot.risk_score,
            hotspot.estimated_complexity,
        ));
        out.push_str(&format!("  signals: {}\n", hotspot.signals.join(", ")));
    }
    out.push('\n');
}

fn render_code_quality_themes(out: &mut String, findings: &[CanonicalFinding]) {
    let mut themes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for finding in findings.iter() {
        themes
            .entry(finding.impact_kind)
            .or_default()
            .push(finding.id.as_str());
    }

    out.push_str("Code Quality Themes\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if themes.is_empty() {
        out.push_str(
            "No broad code-quality themes were synthesized from the current findings.\n\n",
        );
        return;
    }

    for (impact_kind, ids) in themes.into_iter() {
        let preview = ids.iter().take(5).copied().collect::<Vec<_>>().join(", ");
        out.push_str(&format!(
            "{}: {} finding(s) [{}]\n",
            impact_kind,
            ids.len(),
            preview
        ));
    }
    out.push_str(
        "\nInterpret these themes as review clusters: they tell you where multiple findings are reinforcing the same kind of engineering debt or risk surface.\n\n",
    );
}

fn render_remediation_guide(out: &mut String, findings: &[CanonicalFinding]) {
    out.push_str("Remediation Guide\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if findings.is_empty() {
        out.push_str("No remediation items were emitted.\n\n");
        return;
    }

    for finding in findings.iter().take(12) {
        out.push_str(&format!(
            "{} [{}]: {}\n",
            finding.id, finding.classification, finding.remediation
        ));
    }
    out.push('\n');
}

fn render_verification_suggestions(out: &mut String, findings: &[CanonicalFinding]) {
    out.push_str("Verification Suggestions\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if findings.is_empty() {
        out.push_str("No targeted verification suggestions were emitted.\n\n");
        return;
    }

    for finding in findings.iter().take(12) {
        out.push_str(&format!(
            "{} [{}]: {}\n",
            finding.id, finding.impact_kind, finding.verification
        ));
    }
    out.push('\n');
}

fn render_evidence_ledger(out: &mut String, findings: &[CanonicalFinding]) {
    out.push_str("Evidence Ledger\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    if findings.is_empty() {
        out.push_str("No finding evidence was emitted.\n\n");
        return;
    }

    for finding in findings.iter() {
        let ids = evidence_ids(&finding.id, &finding.evidence);
        if ids.is_empty() {
            out.push_str(&format!("{}: no source evidence captured\n", finding.id));
        } else {
            out.push_str(&format!(
                "{}: {} evidence item(s) [{}]\n",
                finding.id,
                ids.len(),
                ids.join(", ")
            ));
        }
    }
    out.push('\n');
}

fn render_conclusion_lenses(
    out: &mut String,
    report: &CrateSourceScanReport,
    findings: &[CanonicalFinding],
) {
    out.push_str("Conclusion Lenses\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!(
        "Rust Maintainer Lens: {}\n",
        rust_maintainer_lens(report, findings)
    ));
    out.push_str(&format!(
        "Compliance Readiness Lens: {}\n",
        compliance_readiness_lens(report, findings)
    ));
    out.push_str(&format!(
        "Certification Preparation Lens: {}\n",
        certification_preparation_lens(report, findings)
    ));
    out.push_str(&format!(
        "Distributed / Operational Lens: {}\n\n",
        distributed_operational_lens(report, findings)
    ));
}

fn conclusion_lenses_json(report: &CrateSourceScanReport, findings: &[CanonicalFinding]) -> Value {
    json!({
        "rustMaintainer": rust_maintainer_lens(report, findings),
        "complianceReadiness": compliance_readiness_lens(report, findings),
        "certificationPreparation": certification_preparation_lens(report, findings),
        "distributedOperational": distributed_operational_lens(report, findings),
    })
}

fn rust_maintainer_lens(report: &CrateSourceScanReport, findings: &[CanonicalFinding]) -> String {
    let immediate = finding_id_preview(findings, |finding| {
        finding.classification == "defect-candidate" || finding.classification == "design-review"
    });
    format!(
        "Use the {:.1}% overall score as a broad code-improvement target, not a compliance or certification badge. The highest-value maintainer work is concentrated in {}.",
        report.certification.audit_score.overall_percent,
        immediate
    )
}

fn compliance_readiness_lens(
    _report: &CrateSourceScanReport,
    findings: &[CanonicalFinding],
) -> String {
    let readiness_count = findings
        .iter()
        .filter(|finding| {
            finding.classification == "review-readiness"
                || finding.classification == "context-needed"
        })
        .count();
    format!(
        "{} finding(s) directly affect analyzability, reproducibility, or review traceability. DSFB may support internal review against standards-oriented expectations, but it does not certify compliance.",
        readiness_count
    )
}

fn certification_preparation_lens(
    _report: &CrateSourceScanReport,
    findings: &[CanonicalFinding],
) -> String {
    let prep = finding_id_preview(findings, |finding| {
        finding.category == "nasa-power-of-ten" || finding.category == "advanced-structural"
    });
    format!(
        "For certification-oriented preparation, treat {} as pre-review cleanup targets and evidence-organizing prompts rather than certification outcomes.",
        prep
    )
}

fn distributed_operational_lens(
    _report: &CrateSourceScanReport,
    findings: &[CanonicalFinding],
) -> String {
    let operational = finding_id_preview(findings, |finding| {
        finding.impact_kind == "concurrency/async" || finding.impact_kind == "resource discipline"
    });
    format!(
        "Operational pressure is most visible in {}. These findings are the most likely to matter later in runtime replay, backpressure review, or production-style load investigation.",
        operational
    )
}

fn finding_id_preview<F>(findings: &[CanonicalFinding], predicate: F) -> String
where
    F: Fn(&CanonicalFinding) -> bool,
{
    let preview = findings
        .iter()
        .filter(|finding| predicate(finding))
        .take(5)
        .map(|finding| finding.id.as_str())
        .collect::<Vec<_>>();
    if preview.is_empty() {
        "no dominant finding cluster in the current report".to_string()
    } else {
        preview.join(", ")
    }
}

fn render_audit_score_table(out: &mut String, score: &AuditScoreCard) {
    let total_checkpoints = score
        .sections
        .iter()
        .map(|section| section.checkpoint_count)
        .sum::<usize>();

    out.push_str("Score Summary Table\n");
    out.push_str("+------------------------------+--------+--------+--------+--------+\n");
    out.push_str("| Section                      | Score% | Weight | Points | Checks |\n");
    out.push_str("+------------------------------+--------+--------+--------+--------+\n");
    for section in &score.sections {
        out.push_str(&format!(
            "| {:<28} | {:>6.1} | {:>6.1} | {:>6.1} | {:>6} |\n",
            truncate_table_label(section.title, 28),
            section.section_percent,
            section.weight_percent,
            section.weighted_points,
            section.checkpoint_count
        ));
    }
    out.push_str("+------------------------------+--------+--------+--------+--------+\n");
    out.push_str(&format!(
        "| {:<28} | {:>6.1} | {:>6.1} | {:>6.1} | {:>6} |\n",
        "Overall",
        score.overall_percent,
        score.possible_weighted_points,
        score.earned_weighted_points,
        total_checkpoints
    ));
    out.push_str("+------------------------------+--------+--------+--------+--------+\n");
}

fn render_advisory_subscore_table(out: &mut String, subscores: &[AdvisorySubscore]) {
    out.push_str("Advisory Broad Subscores\n");
    out.push_str("+------------------------------+--------+\n");
    out.push_str("| Subscore                     | Score% |\n");
    out.push_str("+------------------------------+--------+\n");
    for subscore in subscores.iter() {
        out.push_str(&format!(
            "| {:<28} | {:>6.1} |\n",
            truncate_table_label(subscore.title, 28),
            subscore.percent
        ));
    }
    out.push_str("+------------------------------+--------+\n");
    for subscore in subscores.iter() {
        out.push_str(&format!("  - {}: {}\n", subscore.title, subscore.basis));
    }
}

fn truncate_table_label(label: &str, max_width: usize) -> String {
    let mut out = String::new();
    for ch in label.chars().take(max_width) {
        out.push(ch);
    }
    out
}

fn render_runtime_section(out: &mut String, profile: &RuntimeProfile) {
    out.push_str("Constrained Runtime Profile\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!(
        "no_std declared: {}\n",
        yes_no(profile.no_std_declared)
    ));
    out.push_str(&format!(
        "no_alloc candidate: {}\n",
        yes_no(
            profile.no_std_declared
                && profile.alloc_crate_hits == 0
                && profile.heap_allocation_hits == 0
        )
    ));
    out.push_str(&format!(
        "alloc crate references: {}\n",
        profile.alloc_crate_hits
    ));
    out.push_str(&format!(
        "heap allocation motifs: {}\n",
        profile.heap_allocation_hits
    ));
    render_evidence_block(out, "no_std evidence", &profile.no_std_evidence);
    render_evidence_block(out, "alloc evidence", &profile.alloc_evidence);
    render_evidence_block(
        out,
        "heap-allocation evidence",
        &profile.heap_allocation_evidence,
    );
    out.push('\n');
}

fn render_safety_section(out: &mut String, profile: &SafetyProfile) {
    out.push_str("Unsafe / Panic Surface\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!(
        "unsafe policy: {}\n",
        match profile.unsafe_policy {
            UnsafeCodePolicy::Forbid => "forbid(unsafe_code)",
            UnsafeCodePolicy::Deny => "deny(unsafe_code)",
            UnsafeCodePolicy::NotDeclared => "not declared",
        }
    ));
    out.push_str(&format!(
        "no_unsafe candidate: {}\n",
        yes_no(profile.unsafe_sites == 0)
    ));
    out.push_str(&format!(
        "explicit unsafe sites: {}\n",
        profile.unsafe_sites
    ));
    out.push_str(&format!("panic-like sites: {}\n", profile.panic_sites));
    out.push_str(&format!(
        "unwrap/expect-like sites: {}\n",
        profile.unwrap_sites
    ));
    out.push_str(&format!("FFI boundary sites: {}\n", profile.ffi_sites));
    out.push_str(&format!(
        "SAFETY: justification comments: {}\n",
        profile.safety_comment_sites
    ));
    render_evidence_block(
        out,
        "unsafe policy evidence",
        &profile.unsafe_policy_evidence,
    );
    render_evidence_block(out, "unsafe evidence", &profile.unsafe_evidence);
    render_evidence_block(out, "panic evidence", &profile.panic_evidence);
    render_evidence_block(out, "unwrap evidence", &profile.unwrap_evidence);
    render_evidence_block(out, "FFI evidence", &profile.ffi_evidence);
    render_evidence_block(
        out,
        "SAFETY: comment evidence",
        &profile.safety_comment_evidence,
    );
    out.push('\n');
}

fn render_verification_section(out: &mut String, profile: &VerificationProfile) {
    out.push_str("Verification Evidence Signals\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!(
        "tests/ directory present: {}\n",
        yes_no(profile.tests_dir_present)
    ));
    out.push_str(&format!("test markers: {}\n", profile.test_marker_hits));
    out.push_str(&format!(
        "property-testing signals: {}\n",
        profile.property_testing_hits
    ));
    out.push_str(&format!(
        "concurrency exploration signals: {}\n",
        profile.concurrency_exploration_hits
    ));
    out.push_str(&format!("fuzzing signals: {}\n", profile.fuzzing_hits));
    out.push_str(&format!(
        "formal-method signals: {}\n",
        profile.formal_methods_hits
    ));
    render_evidence_block(out, "test evidence", &profile.test_marker_evidence);
    render_evidence_block(
        out,
        "property-testing evidence",
        &profile.property_testing_evidence,
    );
    render_evidence_block(
        out,
        "concurrency exploration evidence",
        &profile.concurrency_exploration_evidence,
    );
    render_evidence_block(out, "fuzzing evidence", &profile.fuzzing_evidence);
    render_evidence_block(
        out,
        "formal-method evidence",
        &profile.formal_methods_evidence,
    );
    out.push('\n');
}

fn render_build_section(out: &mut String, profile: &BuildProfile) {
    out.push_str("Build / Tooling Complexity\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str(&format!(
        "direct dependencies: {}\n",
        profile.direct_dependencies
    ));
    out.push_str(&format!(
        "build dependencies: {}\n",
        profile.build_dependencies
    ));
    out.push_str(&format!("dev dependencies: {}\n", profile.dev_dependencies));
    out.push_str(&format!(
        "build.rs present: {}\n",
        yes_no(profile.has_build_script)
    ));
    out.push_str(&format!(
        "proc-macro crate: {}\n",
        yes_no(profile.proc_macro_crate)
    ));
    out.push_str(&format!(
        "codegen / native-build signals: {}\n",
        profile.codegen_hits
    ));
    render_evidence_block(out, "codegen evidence", &profile.codegen_evidence);
    out.push('\n');
}

fn render_lifecycle_section(
    out: &mut String,
    profile: &LifecycleProfile,
    manifest: &ManifestMetadata,
) {
    out.push_str("Lifecycle / Governance Artifacts\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    render_lifecycle_presence_lines(out, profile);
    if profile.license_files.is_empty() {
        out.push_str("license files: none observed\n");
    } else {
        let names = profile
            .license_files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("license files: {}\n", names));
    }
    render_manifest_metadata_lines(out, manifest);
    out.push('\n');
}

fn render_lifecycle_presence_lines(out: &mut String, profile: &LifecycleProfile) {
    out.push_str(&format!(
        "README present: {}\n",
        yes_no(profile.readme_present)
    ));
    out.push_str(&format!(
        "CHANGELOG present: {}\n",
        yes_no(profile.changelog_present)
    ));
    out.push_str(&format!(
        "SECURITY.md present: {}\n",
        yes_no(profile.security_md_present)
    ));
    out.push_str(&format!(
        "SAFETY.md present: {}\n",
        yes_no(profile.safety_md_present)
    ));
    out.push_str(&format!(
        "architecture/design doc present: {}\n",
        yes_no(profile.architecture_doc_present)
    ));
    out.push_str(&format!(
        "docs/ content present: {}\n",
        yes_no(profile.docs_dir_present)
    ));
}

fn render_manifest_metadata_lines(out: &mut String, manifest: &ManifestMetadata) {
    out.push_str(&format!(
        "manifest license: {}\n",
        manifest.license.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "manifest rust-version: {}\n",
        manifest.rust_version.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "manifest edition: {}\n",
        manifest.edition.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "repository URL: {}\n",
        manifest.repository.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "documentation URL: {}\n",
        manifest.documentation.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "homepage URL: {}\n",
        manifest.homepage.as_deref().unwrap_or("not declared")
    ));
    out.push_str(&format!(
        "manifest readme: {}\n",
        manifest.readme.as_deref().unwrap_or("not declared")
    ));
}

fn render_power_of_ten_section(out: &mut String, profile: &PowerOfTenProfile) {
    let applied = profile
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::Applied)
        .count();
    let not_applied = profile
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::NotApplied)
        .count();
    let indeterminate = profile
        .rules
        .iter()
        .filter(|rule| rule.status == PowerOfTenStatus::Indeterminate)
        .count();

    out.push_str("NASA/JPL Power of Ten Audit\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    render_power_of_ten_summary(out, applied, not_applied, indeterminate);
    for rule in &profile.rules {
        render_power_of_ten_rule(out, rule);
    }
    out.push('\n');
}

fn render_power_of_ten_summary(
    out: &mut String,
    applied: usize,
    not_applied: usize,
    indeterminate: usize,
) {
    out.push_str(
        "Rust adaptation of Holzmann's Power of Ten rules. C-specific rules are approximated with source-visible Rust proxies. This is guidance for review and improvement, not a certification result.\n",
    );
    out.push_str(&format!(
        "Applied: {} | Not Applied: {} | Indeterminate: {}\n",
        applied, not_applied, indeterminate
    ));
}

fn render_power_of_ten_rule(out: &mut String, rule: &PowerOfTenRuleAudit) {
    out.push_str(&format!(
        "P10-{} {}: {}\n",
        rule.number,
        power_of_ten_status_label(rule.status),
        rule.title
    ));
    out.push_str(&format!("  Detail: {}\n", rule.detail));
    out.push_str(&format!(
        "  Classification: {}\n",
        power_of_ten_classification(rule.number)
    ));
    out.push_str(&format!(
        "  Confidence: {}\n",
        power_of_ten_confidence(rule.status, rule.evidence.len())
    ));
    out.push_str(&format!(
        "  Impact Kind: {}\n",
        power_of_ten_impact_kind(rule.number)
    ));
    out.push_str(&format!(
        "  Why This Matters In Rust: {}\n",
        power_of_ten_rust_why(rule.number)
    ));
    out.push_str(&format!(
        "  Review / Readiness Note: {}\n",
        power_of_ten_readiness_why(rule.number)
    ));
    out.push_str(&format!(
        "  Remediation: {}\n",
        power_of_ten_remediation(rule.number)
    ));
    out.push_str(&format!(
        "  Verification Suggestion: {}\n",
        power_of_ten_verification_suggestion(rule.number)
    ));
    render_named_evidence_block(
        out,
        "  Evidence",
        &format!("P10-{}", rule.number),
        &rule.evidence,
    );
}

fn render_advanced_structural_section(out: &mut String, profile: &AdvancedStructuralProfile) {
    let elevated = profile
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Elevated)
        .count();
    let clear = profile
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Clear)
        .count();
    let indeterminate = profile
        .checks
        .iter()
        .filter(|check| check.status == StructuralCheckStatus::Indeterminate)
        .count();

    out.push_str("Advanced Structural Risk Checks\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    render_advanced_structural_summary(out, elevated, clear, indeterminate);
    for check in &profile.checks {
        render_advanced_structural_check(out, check);
    }
    render_criticality_heatmap(out, &profile.hotspots);
    out.push('\n');
}

fn render_advanced_structural_summary(
    out: &mut String,
    elevated: usize,
    clear: usize,
    indeterminate: usize,
) {
    out.push_str(
        "These checks are source-visible structural proxies for mission, safety, security, and code-quality review. Elevated means review-worthy, not automatically unsafe and not a certification decision.\n",
    );
    out.push_str(&format!(
        "Elevated: {} | Clear: {} | Indeterminate: {}\n",
        elevated, clear, indeterminate
    ));
}

fn render_advanced_structural_check(out: &mut String, check: &AdvancedStructuralCheck) {
    out.push_str(&format!(
        "{} {}: {}\n",
        check.id,
        structural_check_status_label(check.status),
        check.title
    ));
    out.push_str(&format!("  Detail: {}\n", check.detail));
    out.push_str(&format!(
        "  Classification: {}\n",
        advanced_check_classification(check.id)
    ));
    out.push_str(&format!(
        "  Confidence: {}\n",
        advanced_check_confidence(check.status, check.evidence.len())
    ));
    out.push_str(&format!(
        "  Impact Kind: {}\n",
        advanced_check_impact_kind(check.id)
    ));
    out.push_str(&format!(
        "  Why This Matters In Rust: {}\n",
        advanced_check_rust_why(check.id)
    ));
    out.push_str(&format!(
        "  Review / Readiness Note: {}\n",
        advanced_check_readiness_why(check.id)
    ));
    out.push_str(&format!(
        "  Remediation: {}\n",
        advanced_check_remediation(check.id)
    ));
    out.push_str(&format!(
        "  Verification Suggestion: {}\n",
        advanced_check_verification_suggestion(check.id)
    ));
    render_named_evidence_block(out, "  Evidence", check.id, &check.evidence);
}

fn render_criticality_heatmap(out: &mut String, hotspots: &[CriticalityHotspot]) {
    out.push_str("Criticality Heatmap\n");
    out.push_str("──────────────────────────────────────────────────────────────\n");
    out.push_str("Guide:\n");
    out.push_str("  [##--------] observed  score 12-19\n");
    out.push_str("  [####------] guarded   score 20-29\n");
    out.push_str("  [######----] elevated  score 30-39\n");
    out.push_str("  [########--] high      score 40-49\n");
    out.push_str("  [##########] severe    score 50+\n");
    out.push_str("  row format: path:line `function` [bar] band score=<n> complexity~<n>\n");
    out.push_str("  signals: comma-separated structural risk contributors\n");
    if hotspots.is_empty() {
        out.push_str("No function hotspots extracted.\n");
        return;
    }

    for hotspot in hotspots {
        out.push_str(&format!(
            "{}:{} `{}` {} {:<8} score={} complexity~={}\n",
            hotspot.path.display(),
            hotspot.start_line,
            hotspot.function_name,
            heatmap_bar(hotspot.risk_score),
            heatmap_band_label(hotspot.risk_score),
            hotspot.risk_score,
            hotspot.estimated_complexity,
        ));
        out.push_str(&format!("  signals: {}\n", hotspot.signals.join(", ")));
    }
}

fn render_evidence_block(out: &mut String, label: &str, evidence: &[ScanEvidence]) {
    if evidence.is_empty() {
        return;
    }
    out.push_str(&format!("{label}:\n"));
    for item in evidence.iter() {
        out.push_str(&format!(
            "  - {}:{} [{}] {}\n",
            item.path.display(),
            item.line_number,
            item.pattern,
            item.snippet
        ));
    }
}

fn render_named_evidence_block(
    out: &mut String,
    label: &str,
    finding_id: &str,
    evidence: &[ScanEvidence],
) {
    if evidence.is_empty() {
        return;
    }
    out.push_str(&format!("{label}:\n"));
    for (idx, item) in evidence.iter().enumerate() {
        out.push_str(&format!(
            "  - {} {}:{} [{}] {}\n",
            evidence_id(finding_id, item, idx),
            item.path.display(),
            item.line_number,
            item.pattern,
            item.snippet
        ));
    }
}

fn heatmap_band_label(score: usize) -> &'static str {
    match score {
        0..=19 => "observed",
        20..=29 => "guarded",
        30..=39 => "elevated",
        40..=49 => "high",
        _other => "severe",
    }
}

fn heatmap_bar(score: usize) -> String {
    let filled = match score {
        0..=19 => 2,
        20..=29 => 4,
        30..=39 => 6,
        40..=49 => 8,
        _other => 10,
    };

    let mut out = String::with_capacity(12);
    out.push('[');
    for idx in 0..10 {
        out.push(if idx < filled { '#' } else { '-' });
    }
    out.push(']');
    out
}

fn power_of_ten_status_label(status: PowerOfTenStatus) -> &'static str {
    match status {
        PowerOfTenStatus::Applied => "applied",
        PowerOfTenStatus::NotApplied => "not applied",
        PowerOfTenStatus::Indeterminate => "indeterminate",
    }
}

fn structural_check_status_label(status: StructuralCheckStatus) -> &'static str {
    match status {
        StructuralCheckStatus::Elevated => "elevated",
        StructuralCheckStatus::Clear => "clear",
        StructuralCheckStatus::Indeterminate => "indeterminate",
    }
}

fn power_of_ten_remediation(rule_number: u8) -> &'static str {
    match rule_number {
        1 => "Remove recursion where possible, or isolate the pattern behind a bounded proof and explicit review note.",
        2 => "Add explicit upper bounds, timeout guards, or fixed-step limits so loop behavior is reviewable.",
        3 => "Move dynamic allocation to initialization paths or document and bound the steady-state allocation sites.",
        4 => "Split large functions into reviewable units with clearer local invariants and narrower responsibilities.",
        5 => "Replace catch-all control flow with explicit state handling or document the fallback state as intentional.",
        6 => "Reduce dependence on global mutable state or document synchronization and ownership boundaries.",
        7 => "Propagate errors explicitly rather than unwrapping, or document the invariant that justifies the unwrap/expect.",
        8 => "Reduce conditional-compilation forks or document why each feature/macro path remains auditable.",
        9 => "Tighten raw-pointer / FFI surfaces and document the local safety contract for each remaining site.",
        10 => "Keep warnings and analyzer gates active in CI so the audit surface stays reviewable over time.",
        _other => "Review the flagged rule against the locked DSFB audit guidance and simplify the local structure where practical.",
    }
}

fn advanced_check_remediation(check_id: &str) -> &'static str {
    match check_id {
        "SAFE-STATE" => "Make fallback states explicit and document what the safe-state behavior is for the affected control path.",
        "ASYNC-LOCK" => "Avoid holding locks across `.await`, or split the critical section so async suspension happens after the guard is dropped.",
        "ALLOC-HOT" => "Pre-allocate, bound growth, or move allocation-heavy work out of high-frequency loops.",
        "CLOCK-INTEG" => "Prefer monotonic time for durations and timeout logic; isolate wall-clock usage to presentation or signed timestamps.",
        "RETRY-DAMP" => "Add capped exponential backoff with jitter and make retry behavior explicit in failure-path tests.",
        "HARD-WAIT" => "Replace fixed sleeps with state checks, deadlines, or explicit readiness conditions when possible.",
        "PART-SPACE" => "Reduce shared global state, or document the partitioning/ownership rationale for any remaining shared resource.",
        "PLUGIN-LOAD" => "Constrain dynamic loading behind verification, sandboxing, or explicit operator review.",
        "CWE-404" => "Tighten ownership so resources close on all error paths and avoid raw-handle escape hatches unless documented.",
        "CMD-BUF" => "Add TTL, sequence, staleness, or cancellation guards so queued control messages cannot accumulate invisibly.",
        "ITER-UNB" => "Add `.take(...)`, explicit bounds, or documented finite-source guarantees on terminal iterator consumption.",
        "ISR-SAFE" => "Keep interrupt handlers allocation-free and lock-free where possible, or document the ISR contract explicitly.",
        "FUTURE-WAKE" => "Ensure every manual `Poll::Pending` path arranges a wakeup before returning pending.",
        "TASK-LEAK" => "Retain JoinHandles, cancellation paths, or supervision ownership for spawned tasks that affect shutdown and backpressure.",
        "DROP-PANIC" => "Keep `Drop` implementations infallible; move failure reporting out of destructor paths.",
        "ATOMIC-RELAXED" => "Review whether the flagged atomic needs stronger ordering semantics on the observed state-transition path.",
        "CLOCK-MIX" => "Avoid mixing `Instant` and `SystemTime` in one duration/control path unless the conversion boundary is explicit.",
        "SHORT-WRITE" => "Use `write_all`, retry `Interrupted`, or document why partial writes are already handled by the caller.",
        "ASYNC-RECUR" => "Add a visible base-case/depth bound or replace async recursion with an explicit work queue or loop.",
        "CHAN-UNB" => "Prefer bounded channels or document why unbounded growth is safe under the expected ingress rate.",
        "ZERO-COPY" => "Keep ingress data borrowed or reference-counted longer, and avoid eager `.to_vec()` / `.clone()` on hot read paths.",
        "CARGO-VERS" => "Pin or narrow dependency version requirements so builds and attestations remain reproducible.",
        _other => "Review the finding against the emitted evidence and either tighten the local structure or document the local invariant.",
    }
}

fn heuristic_remediation(heuristic_id: &str) -> &'static str {
    match heuristic_id {
        "H-ALLOC-01" => "Audit hot-loop allocation sites and prefer bounded or reserved growth on steady-state paths.",
        "H-LOCK-01" => "Review lock hold time, await-under-lock risk, and whether the shared state can be partitioned or copied out earlier.",
        "H-RAFT-01" => "Check heartbeat timeout logic, monotonic time usage, and whether election-sensitive paths have explicit headroom.",
        "H-ASYNC-01" => "Move blocking work out of async tasks or add explicit offload/yield boundaries.",
        "H-TCP-01" => "Review partial-write handling, retry damping, timeout paths, and whether network assumptions are made explicit.",
        "H-CHAN-01" => "Check boundedness, receiver saturation handling, and whether producers can observe downstream backpressure.",
        "H-CLOCK-01" => "Prefer monotonic clocks for control logic and isolate wall-clock use to presentation or external protocol boundaries.",
        "H-THRU-01" => "Inspect hot paths for hidden copies, queue growth, or retry behavior that can erode throughput before alarms fire.",
        "H-SERDE-01" => "Review payload growth, eager allocation, and schema-boundary handling on the serialization path.",
        "H-GRPC-01" => "Inspect flow-control behavior, buffering, and async fairness on the affected RPC path.",
        "H-DNS-01" => "Review cache invalidation, timeout handling, and fallback behavior around name resolution.",
        "H-ERR-01" => "Audit retry policy, escalation path, and whether repeated failure surfaces are bounded and jittered.",
        _other => "Review the matched motif against its evidence and either tighten the local structure or record the intended invariant.",
    }
}

fn evidence_ids(finding_id: &str, evidence: &[ScanEvidence]) -> Vec<String> {
    evidence
        .iter()
        .enumerate()
        .map(|(idx, item)| evidence_id(finding_id, item, idx))
        .collect()
}

fn evidence_id(finding_id: &str, evidence: &ScanEvidence, index: usize) -> String {
    format!(
        "{}-{:02}-{}-{}",
        finding_id,
        index + 1,
        sanitize_evidence_component(
            evidence
                .path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("file")
        ),
        evidence.line_number
    )
}

fn sanitize_evidence_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn scan_runtime_profile(documents: &[SourceDocument]) -> RuntimeProfile {
    let no_std = scan_patterns(documents, NO_STD_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let alloc = scan_patterns(documents, ALLOC_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let heap = scan_patterns(documents, HEAP_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let runtime_core_alloc = scan_patterns_filtered(
        documents,
        ALLOC_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
        ScanContentMode::Risk,
        |path| !is_tooling_support_path(path),
    );
    let runtime_core_heap = scan_patterns_filtered(
        documents,
        HEAP_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
        ScanContentMode::Risk,
        |path| !is_tooling_support_path(path),
    );

    RuntimeProfile {
        no_std_declared: no_std.total_hits > 0,
        no_std_evidence: no_std.evidence,
        alloc_crate_hits: alloc.total_hits,
        alloc_evidence: alloc.evidence,
        heap_allocation_hits: heap.total_hits,
        heap_allocation_evidence: heap.evidence,
        runtime_core_alloc_hits: runtime_core_alloc.total_hits,
        runtime_core_heap_allocation_hits: runtime_core_heap.total_hits,
    }
}

fn scan_safety_profile(documents: &[SourceDocument]) -> SafetyProfile {
    let forbid = scan_patterns(documents, FORBID_UNSAFE_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let deny = scan_patterns(documents, DENY_UNSAFE_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let unsafe_scan = scan_patterns(documents, UNSAFE_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let panic_scan = scan_patterns(documents, PANIC_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let unwrap_scan = scan_patterns(documents, UNWRAP_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let ffi_scan = scan_patterns(documents, FFI_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let safety_comment_scan =
        scan_patterns(documents, SAFETY_COMMENT_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);

    let (unsafe_policy, unsafe_policy_evidence) = if forbid.total_hits > 0 {
        (UnsafeCodePolicy::Forbid, forbid.evidence)
    } else if deny.total_hits > 0 {
        (UnsafeCodePolicy::Deny, deny.evidence)
    } else {
        (UnsafeCodePolicy::NotDeclared, Vec::new())
    };

    SafetyProfile {
        unsafe_policy,
        unsafe_policy_evidence,
        unsafe_sites: unsafe_scan.total_hits,
        unsafe_evidence: unsafe_scan.evidence,
        panic_sites: panic_scan.total_hits,
        panic_evidence: panic_scan.evidence,
        unwrap_sites: unwrap_scan.total_hits,
        unwrap_evidence: unwrap_scan.evidence,
        ffi_sites: ffi_scan.total_hits,
        ffi_evidence: ffi_scan.evidence,
        safety_comment_sites: safety_comment_scan.total_hits,
        safety_comment_evidence: safety_comment_scan.evidence,
    }
}

fn scan_verification_profile(
    all_files: &[PathBuf],
    documents: &[SourceDocument],
) -> VerificationProfile {
    let tests_dir_present = all_files
        .iter()
        .any(|path| has_path_component(path, "tests"));
    let test_scan = scan_patterns_analysis(documents, TEST_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let property_scan =
        scan_patterns_analysis(documents, PROPERTY_TEST_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let concurrency_scan = scan_patterns_analysis(
        documents,
        CONCURRENCY_EXPLORATION_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    let fuzzing_scan = scan_patterns_analysis(documents, FUZZING_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let formal_scan =
        scan_patterns_analysis(documents, FORMAL_METHOD_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);

    VerificationProfile {
        tests_dir_present,
        test_marker_hits: test_scan.total_hits,
        test_marker_evidence: test_scan.evidence,
        property_testing_hits: property_scan.total_hits,
        property_testing_evidence: property_scan.evidence,
        concurrency_exploration_hits: concurrency_scan.total_hits,
        concurrency_exploration_evidence: concurrency_scan.evidence,
        fuzzing_hits: fuzzing_scan.total_hits,
        fuzzing_evidence: fuzzing_scan.evidence,
        formal_methods_hits: formal_scan.total_hits,
        formal_methods_evidence: formal_scan.evidence,
    }
}

fn scan_build_profile(
    root: &Path,
    documents: &[SourceDocument],
    manifest: &ManifestMetadata,
) -> BuildProfile {
    let codegen_scan = scan_patterns(documents, CODEGEN_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let has_build_script = manifest.build_script.is_some() || root.join("build.rs").is_file();

    BuildProfile {
        direct_dependencies: manifest.direct_dependencies,
        build_dependencies: manifest.build_dependencies,
        dev_dependencies: manifest.dev_dependencies,
        has_build_script,
        proc_macro_crate: manifest.proc_macro,
        codegen_hits: codegen_scan.total_hits,
        codegen_evidence: codegen_scan.evidence,
    }
}

fn scan_lifecycle_profile(all_files: &[PathBuf]) -> LifecycleProfile {
    let mut license_files = Vec::new();

    for path in all_files {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let lowered = name.to_ascii_lowercase();
        if lowered.starts_with("license")
            || lowered.starts_with("copying")
            || lowered.starts_with("notice")
        {
            license_files.push(path.clone());
        }
    }

    license_files.sort();

    LifecycleProfile {
        readme_present: has_file_with_prefix(all_files, "readme"),
        changelog_present: has_file_with_prefix(all_files, "changelog")
            || has_file_with_prefix(all_files, "changes"),
        security_md_present: has_exact_file_name(all_files, "security.md"),
        safety_md_present: has_exact_file_name(all_files, "safety.md"),
        architecture_doc_present: has_exact_file_name(all_files, "architecture.md")
            || has_exact_file_name(all_files, "design.md"),
        docs_dir_present: all_files
            .iter()
            .any(|path| has_path_component(path, "docs")),
        license_files,
    }
}

fn scan_power_of_ten_profile(
    documents: &[SourceDocument],
    artifact_documents: &[SourceDocument],
    functions: &[FunctionSummary],
    runtime: &RuntimeProfile,
    safety: &SafetyProfile,
    build: &BuildProfile,
) -> PowerOfTenProfile {
    PowerOfTenProfile {
        rules: vec![
            build_power_of_ten_rule1(documents, functions),
            build_power_of_ten_rule2(documents),
            build_power_of_ten_rule3(runtime),
            build_power_of_ten_rule4(functions),
            build_power_of_ten_rule5(functions),
            build_power_of_ten_rule6(documents),
            build_power_of_ten_rule7(documents, safety),
            build_power_of_ten_rule8(documents, build),
            build_power_of_ten_rule9(documents),
            build_power_of_ten_rule10(artifact_documents),
        ],
    }
}

fn scan_advanced_structural_profile(
    documents: &[SourceDocument],
    artifact_documents: &[SourceDocument],
    functions: &[FunctionSummary],
    safety: &SafetyProfile,
) -> AdvancedStructuralProfile {
    AdvancedStructuralProfile {
        checks: vec![
            build_recursion_check(functions),
            build_interior_mutability_check(documents),
            build_unwrap_safety_check(safety),
            build_complexity_check(functions),
            build_async_lock_check(documents),
            build_safe_state_check(documents),
            build_time_wait_check(documents),
            build_partition_space_check(documents),
            build_plugin_load_check(artifact_documents),
            build_resource_lifecycle_check(documents),
            build_command_buffer_check(documents),
            build_iterator_bound_check(functions),
            build_isr_safety_check(functions),
            build_future_wake_check(functions),
            build_task_leak_check(documents),
            build_drop_panic_check(documents),
            build_relaxed_atomic_check(functions),
            build_clock_mix_check(functions),
            build_short_write_check(functions),
            build_async_recursion_check(functions),
            build_unbounded_channel_check(documents),
            build_zero_copy_check(functions),
            build_version_drift_check(documents),
        ],
        hotspots: build_criticality_hotspots(functions),
    }
}

fn build_power_of_ten_rule1(
    documents: &[SourceDocument],
    functions: &[FunctionSummary],
) -> PowerOfTenRuleAudit {
    let rule1_scan = scan_patterns(documents, P10_RULE1_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let (rule1_recursion_hits, rule1_recursion_evidence) =
        collect_direct_recursion_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    let total_hits = rule1_scan.total_hits + rule1_recursion_hits;
    let mut evidence = rule1_scan.evidence;
    append_evidence(
        &mut evidence,
        rule1_recursion_evidence,
        MAX_EVIDENCE_PER_SIGNAL,
    );

    PowerOfTenRuleAudit {
        number: 1,
        title: "Simple control flow; no recursion or equivalent escapes",
        status: if total_hits == 0 {
            PowerOfTenStatus::Applied
        } else {
            PowerOfTenStatus::NotApplied
        },
        detail: if total_hits == 0 {
            "No direct recursion or obvious control-flow escape motifs observed. Indirect recursion is not proven absent by this lightweight scan.".to_string()
        } else {
            format!(
                "{total_hits} direct-recursion site(s) or control-flow escape motif(s) observed."
            )
        },
        evidence,
    }
}

fn build_power_of_ten_rule2(documents: &[SourceDocument]) -> PowerOfTenRuleAudit {
    let rule2_unbounded = collect_unbounded_loop_evidence(documents, MAX_EVIDENCE_PER_SIGNAL);
    let (ambiguous_for_hits, ambiguous_for_evidence) =
        collect_ambiguous_for_loop_evidence(documents, MAX_EVIDENCE_PER_SIGNAL);
    let (status, detail, evidence) = if rule2_unbounded.total_hits > 0 {
        (
            PowerOfTenStatus::NotApplied,
            format!(
                "{} potentially unbounded `loop`/`while` construct(s) observed.",
                rule2_unbounded.total_hits
            ),
            rule2_unbounded.evidence,
        )
    } else if ambiguous_for_hits > 0 {
        (
            PowerOfTenStatus::Indeterminate,
            format!(
                "{} `for` loop site(s) remain ambiguous after bounded-iterator screening.",
                ambiguous_for_hits
            ),
            ambiguous_for_evidence,
        )
    } else {
        (
            PowerOfTenStatus::Applied,
            "No unbounded loops or ambiguous iterator-driven `for` loops were observed."
                .to_string(),
            Vec::new(),
        )
    };

    PowerOfTenRuleAudit {
        number: 2,
        title: "All loops have a fixed upper bound",
        status,
        detail,
        evidence,
    }
}

fn build_power_of_ten_rule3(runtime: &RuntimeProfile) -> PowerOfTenRuleAudit {
    let total_hits = runtime.alloc_crate_hits + runtime.heap_allocation_hits;
    let runtime_hits = runtime.runtime_core_alloc_hits + runtime.runtime_core_heap_allocation_hits;
    let (status, detail) = if total_hits == 0 {
        (
            PowerOfTenStatus::Applied,
            "No heap-allocation motifs observed.".to_string(),
        )
    } else if runtime_hits == 0 {
        (
            PowerOfTenStatus::Applied,
            format!(
                "No runtime-core heap-allocation motifs were observed. {total_hits} allocation motif(s) remain in tooling, reporting, evaluation, or other non-runtime-support paths."
            ),
        )
    } else {
        (
            PowerOfTenStatus::NotApplied,
            format!(
                "{total_hits} heap-allocation motif(s) observed, including {runtime_hits} runtime-core signal(s). This crate-level scan cannot distinguish initialization-only allocation from steady-state allocation."
            ),
        )
    };

    PowerOfTenRuleAudit {
        number: 3,
        title: "No dynamic allocation after initialization",
        status,
        detail,
        evidence: runtime.heap_allocation_evidence.clone(),
    }
}

fn build_power_of_ten_rule4(functions: &[FunctionSummary]) -> PowerOfTenRuleAudit {
    let evidence = collect_long_function_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    let long_function_count = functions
        .iter()
        .filter(|function| function.line_count > 60)
        .count();

    PowerOfTenRuleAudit {
        number: 4,
        title: "Functions stay within a single-sheet size budget (~60 LOC)",
        status: if long_function_count == 0 {
            PowerOfTenStatus::Applied
        } else {
            PowerOfTenStatus::NotApplied
        },
        detail: if long_function_count == 0 {
            "No function over 60 lines was observed by the scanner.".to_string()
        } else {
            format!("{long_function_count} function(s) exceed the 60-line threshold.")
        },
        evidence,
    }
}

fn build_power_of_ten_rule5(functions: &[FunctionSummary]) -> PowerOfTenRuleAudit {
    let total_assertions = functions
        .iter()
        .map(|function| function.assertion_count)
        .sum::<usize>();
    let avg_assertions = if functions.is_empty() {
        0.0
    } else {
        total_assertions as f64 / functions.len() as f64
    };

    PowerOfTenRuleAudit {
        number: 5,
        title: "Assertion density averages at least two per function",
        status: if functions.is_empty() {
            PowerOfTenStatus::Indeterminate
        } else if avg_assertions >= 2.0 {
            PowerOfTenStatus::Applied
        } else {
            PowerOfTenStatus::NotApplied
        },
        detail: if functions.is_empty() {
            "No function bodies were extracted, so assertion density could not be estimated."
                .to_string()
        } else {
            format!(
                "Estimated assertion density is {:.2} per function across {} extracted function(s).",
                avg_assertions,
                functions.len()
            )
        },
        evidence: collect_low_assertion_evidence(functions, MAX_EVIDENCE_PER_SIGNAL),
    }
}

fn build_power_of_ten_rule6(documents: &[SourceDocument]) -> PowerOfTenRuleAudit {
    let scan = scan_global_shared_resource_patterns(documents, MAX_EVIDENCE_PER_SIGNAL);
    PowerOfTenRuleAudit {
        number: 6,
        title: "Data objects remain at the smallest practical scope",
        status: if scan.total_hits == 0 {
            PowerOfTenStatus::Applied
        } else {
            PowerOfTenStatus::NotApplied
        },
        detail: if scan.total_hits == 0 {
            "No obvious crate-global mutable/shared state motifs were observed. This is only a proxy for scope minimization.".to_string()
        } else {
            format!(
                "{} crate-global mutable/shared state motif(s) observed.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_power_of_ten_rule7(
    documents: &[SourceDocument],
    safety: &SafetyProfile,
) -> PowerOfTenRuleAudit {
    let scan = scan_patterns(
        documents,
        P10_RULE7_EXPLICIT_IGNORE_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    let (status, detail) = if scan.total_hits > 0 || safety.unwrap_sites > 0 {
        (
            PowerOfTenStatus::NotApplied,
            format!(
                "{} explicit discard site(s) and {} unwrap/expect site(s) observed. Parameter validation cannot be proven by this scan.",
                scan.total_hits, safety.unwrap_sites
            ),
        )
    } else {
        (
            PowerOfTenStatus::Indeterminate,
            "No obvious unchecked-return motifs were observed, but parameter validation and full return-value propagation are not mechanically proven by this scanner."
                .to_string(),
        )
    };
    let mut evidence = scan.evidence;
    append_evidence(
        &mut evidence,
        safety.unwrap_evidence.clone(),
        MAX_EVIDENCE_PER_SIGNAL,
    );

    PowerOfTenRuleAudit {
        number: 7,
        title: "Return values are checked and parameters are validated",
        status,
        detail,
        evidence,
    }
}

fn build_power_of_ten_rule8(
    documents: &[SourceDocument],
    build: &BuildProfile,
) -> PowerOfTenRuleAudit {
    let rule8_cfg = scan_cfg_surface(documents, MAX_EVIDENCE_PER_SIGNAL);
    let rule8_macro = scan_patterns(documents, P10_RULE8_MACRO_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let mut evidence = rule8_cfg.evidence;
    append_evidence(&mut evidence, rule8_macro.evidence, MAX_EVIDENCE_PER_SIGNAL);

    PowerOfTenRuleAudit {
        number: 8,
        title: "Conditional compilation and metaprogramming stay minimal",
        status: if build.proc_macro_crate || rule8_macro.total_hits > 0 || rule8_cfg.total_hits > 12
        {
            PowerOfTenStatus::NotApplied
        } else if rule8_cfg.total_hits > 4 {
            PowerOfTenStatus::Indeterminate
        } else {
            PowerOfTenStatus::Applied
        },
        detail: format!(
            "{} review-relevant conditional-compilation site(s), {} macro-definition/proc-macro site(s) observed. This is a Rust adaptation of the C preprocessor rule.",
            rule8_cfg.total_hits,
            rule8_macro.total_hits
        ),
        evidence,
    }
}

fn build_power_of_ten_rule9(documents: &[SourceDocument]) -> PowerOfTenRuleAudit {
    let scan = scan_restricted_pointer_use(documents, MAX_EVIDENCE_PER_SIGNAL);
    PowerOfTenRuleAudit {
        number: 9,
        title: "Pointer use remains restricted",
        status: if scan.total_hits == 0 {
            PowerOfTenStatus::Applied
        } else {
            PowerOfTenStatus::NotApplied
        },
        detail: if scan.total_hits == 0 {
            "No raw-pointer or function-pointer motifs were observed.".to_string()
        } else {
            format!(
                "{} raw-pointer/function-pointer motif(s) observed.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_power_of_ten_rule10(artifact_documents: &[SourceDocument]) -> PowerOfTenRuleAudit {
    let rule10_warnings = scan_patterns(
        artifact_documents,
        P10_RULE10_WARNING_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    let rule10_analyzers = scan_patterns(
        artifact_documents,
        P10_RULE10_ANALYZER_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    let mut evidence = rule10_warnings.evidence;
    append_evidence(
        &mut evidence,
        rule10_analyzers.evidence,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    let (status, detail) = if rule10_warnings.total_hits > 0 && rule10_analyzers.total_hits > 0 {
        (
            PowerOfTenStatus::Applied,
            format!(
                "Observed warning-strictness signal(s) ({}) and static-analysis signal(s) ({}). Daily cadence and zero-warning status are not provable from packaged crate sources.",
                rule10_warnings.total_hits, rule10_analyzers.total_hits
            ),
        )
    } else if rule10_warnings.total_hits > 0 || rule10_analyzers.total_hits > 0 {
        (
            PowerOfTenStatus::Indeterminate,
            format!(
                "Observed warning/analyzer signal(s), but the full Power-of-Ten requirement for pedantic warnings plus regular analyzer use is not established. Warning signals: {}, analyzer signals: {}.",
                rule10_warnings.total_hits, rule10_analyzers.total_hits
            ),
        )
    } else {
        (
            PowerOfTenStatus::NotApplied,
            "No warning-strictness or static-analyzer signals were observed in the packaged crate artifacts. This may under-report projects whose CI metadata is not published with the crate.".to_string(),
        )
    };

    PowerOfTenRuleAudit {
        number: 10,
        title: "Pedantic warnings and static analyzers are enforced",
        status,
        detail,
        evidence,
    }
}

fn build_recursion_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (direct_recursion_hits, direct_recursion_evidence) =
        collect_direct_recursion_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    let (indirect_cycle_hits, indirect_cycle_evidence) =
        collect_indirect_recursion_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    let total_hits = direct_recursion_hits + indirect_cycle_hits;
    let mut evidence = direct_recursion_evidence;
    append_evidence(
        &mut evidence,
        indirect_cycle_evidence,
        MAX_EVIDENCE_PER_SIGNAL,
    );

    AdvancedStructuralCheck {
        id: "JPL-R0",
        title: "Recursion and cyclic call graph audit",
        status: if total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if total_hits == 0 {
            "No direct recursion or local call-cycle motifs were observed.".to_string()
        } else {
            format!(
                "{} direct-recursion hit(s) and {} local indirect cycle(s) observed.",
                direct_recursion_hits, indirect_cycle_hits
            )
        },
        evidence,
    }
}

fn build_interior_mutability_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(
        documents,
        INTERIOR_MUTABILITY_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    AdvancedStructuralCheck {
        id: "JPL-R4",
        title: "Data-flow traceability / interior mutability audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No interior-mutability motifs were observed.".to_string()
        } else {
            format!(
                "{} interior-mutability motif(s) observed (Cell/RefCell/UnsafeCell/atomic types).",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_unwrap_safety_check(safety: &SafetyProfile) -> AdvancedStructuralCheck {
    AdvancedStructuralCheck {
        id: "JPL-R9",
        title: "Unchecked extraction / dereference safety audit",
        status: if safety.unwrap_sites == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if safety.unwrap_sites == 0 {
            "No unwrap/expect extraction sites were observed.".to_string()
        } else {
            format!(
                "{} unwrap/expect-like site(s) observed; these deserve explicit invariant review in high-assurance code.",
                safety.unwrap_sites
            )
        },
        evidence: safety.unwrap_evidence.clone(),
    }
}

fn build_complexity_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let complexity_hotspots = collect_complexity_hotspots(functions, MAX_EVIDENCE_PER_SIGNAL);
    let exceedances = complexity_hotspots
        .iter()
        .filter(|entry| entry.estimated_complexity > 15)
        .count();
    let evidence = complexity_hotspots
        .iter()
        .take(MAX_EVIDENCE_PER_SIGNAL)
        .map(|entry| ScanEvidence {
            path: entry.path.clone(),
            line_number: entry.start_line,
            pattern: "estimated cyclomatic complexity",
            snippet: format!(
                "function `{}` has estimated complexity {}",
                entry.function_name, entry.estimated_complexity
            ),
        })
        .collect();

    AdvancedStructuralCheck {
        id: "NASA-CC",
        title: "Cyclomatic complexity hotspot audit (NASA SWE-220 proxy)",
        status: if exceedances > 0 {
            StructuralCheckStatus::Elevated
        } else if complexity_hotspots.is_empty() {
            StructuralCheckStatus::Indeterminate
        } else {
            StructuralCheckStatus::Clear
        },
        detail: if complexity_hotspots.is_empty() {
            "No function summaries were extracted, so complexity could not be estimated."
                .to_string()
        } else {
            format!(
                "{} extracted hotspot(s); {} exceed the NASA safety-critical threshold of 15 by this lightweight estimate.",
                complexity_hotspots.len(),
                exceedances
            )
        },
        evidence,
    }
}

fn build_async_lock_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(documents, ASYNC_LOCK_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "H-ASYNC-LOCK",
        title: "Async lock contention / priority inversion proxy",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No async lock contention motifs were observed.".to_string()
        } else {
            format!(
                "{} async lock motif(s) observed. This is a priority-inversion proxy, not proof of scheduler-level priority inversion.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_safe_state_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(documents, CATCH_ALL_MATCH_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "SAFE-STATE",
        title: "Catch-all state handling / safe-state fallback audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No `_ =>` catch-all state transitions were observed.".to_string()
        } else {
            format!(
                "{} catch-all match arm(s) observed; explicit state enumeration is preferable for safety review.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_time_wait_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(documents, HARD_CODED_WAIT_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "TIME-WAIT",
        title: "Hard-coded timing assumption audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No hard-coded sleep/timing-wait motifs were observed.".to_string()
        } else {
            format!(
                "{} hard-coded wait motif(s) observed. Review whether these are deterministic control waits or deadline-free timing assumptions.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_partition_space_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_global_shared_resource_patterns(documents, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "PART-SPACE",
        title: "Global shared-resource / partitioning-risk audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No obvious global shared-resource motifs were observed.".to_string()
        } else {
            format!(
                "{} global shared-resource motif(s) observed.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_plugin_load_check(artifact_documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(
        artifact_documents,
        DYNAMIC_LOADING_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    AdvancedStructuralCheck {
        id: "PLUGIN-LOAD",
        title: "Dynamic loading / plugin sandbox audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No dynamic loading motifs were observed.".to_string()
        } else {
            format!("{} dynamic loading motif(s) observed.", scan.total_hits)
        },
        evidence: scan.evidence,
    }
}

fn build_resource_lifecycle_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(
        documents,
        RESOURCE_LIFECYCLE_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    AdvancedStructuralCheck {
        id: "CWE-404",
        title: "Manual resource-lifecycle / shutdown audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No manual resource-lifecycle motifs were observed.".to_string()
        } else {
            format!(
                "{} manual lifecycle motif(s) observed (raw handles, forget, ManuallyDrop, mmap).",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_command_buffer_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let command_scan = scan_patterns(documents, COMMAND_BUFFER_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let ttl_scan = scan_patterns(documents, TTL_GUARD_PATTERNS, MAX_EVIDENCE_PER_SIGNAL);
    let mut evidence = command_scan.evidence;
    append_evidence(&mut evidence, ttl_scan.evidence, MAX_EVIDENCE_PER_SIGNAL);
    let (status, detail) = if command_scan.total_hits == 0 {
        (
            StructuralCheckStatus::Clear,
            "No command/control queue motifs were observed.".to_string(),
        )
    } else if ttl_scan.total_hits == 0 {
        (
            StructuralCheckStatus::Elevated,
            format!(
                "{} command/control queue motif(s) observed without TTL/staleness/sequence guard signals.",
                command_scan.total_hits
            ),
        )
    } else {
        (
            StructuralCheckStatus::Indeterminate,
            format!(
                "{} command/control queue motif(s) and {} freshness-guard motif(s) observed.",
                command_scan.total_hits, ttl_scan.total_hits
            ),
        )
    };

    AdvancedStructuralCheck {
        id: "CMD-BUF",
        title: "Hazardous command buffering audit",
        status,
        detail,
        evidence,
    }
}

fn build_iterator_bound_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_unbounded_iterator_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "ITER-UNB",
        title: "Unbounded iterator terminal-consumption audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No iterator terminal-consumption sites lacking an obvious `.take()` bound were observed."
                .to_string()
        } else {
            format!(
                "{hits} iterator terminal site(s) use collect/fold/count/last/sum without an obvious `.take()` or single-step bound."
            )
        },
        evidence,
    }
}

fn build_isr_safety_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_interrupt_safety_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "ISR-SAFE",
        title: "Interrupt-context allocation / lock audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No interrupt handlers with allocation or mutex/lock motifs were observed.".to_string()
        } else {
            format!(
                "{hits} interrupt-context site(s) contain allocation or lock motifs that deserve ISR safety review."
            )
        },
        evidence,
    }
}

fn build_future_wake_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) =
        collect_pending_without_waker_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "FUTURE-WAKE",
        title: "Manual Future pending-without-waker audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No manual `Poll::Pending` sites without local wake registration were observed."
                .to_string()
        } else {
            format!(
                "{hits} manual poll function(s) return `Poll::Pending` without an obvious waker registration motif."
            )
        },
        evidence,
    }
}

fn build_task_leak_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_join_handle_discard(documents, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "TASK-LEAK",
        title: "Detached-task / discarded JoinHandle audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No explicit discarded Tokio JoinHandle sites were observed.".to_string()
        } else {
            format!(
                "{} Tokio spawn/spawn_blocking site(s) appear to discard the JoinHandle explicitly.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_drop_panic_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_panic_in_drop_evidence(documents, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "DROP-PANIC",
        title: "Panic-in-Drop audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No panic-like sites were observed inside `impl Drop` bodies.".to_string()
        } else {
            format!("{hits} panic-like site(s) were observed inside `impl Drop` bodies.")
        },
        evidence,
    }
}

fn build_relaxed_atomic_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) =
        collect_relaxed_atomic_state_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "ATOMIC-RELAXED",
        title: "Relaxed atomic ordering on critical-state paths",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No `Ordering::Relaxed` sites were observed on functions that also look like critical state-transition logic."
                .to_string()
        } else {
            format!(
                "{hits} function(s) combine `Ordering::Relaxed` with consensus/state-transition motifs."
            )
        },
        evidence,
    }
}

fn build_clock_mix_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_mixed_clock_source_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "CLOCK-MIX",
        title: "Mixed monotonic/wall-clock duration audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No functions were observed mixing `Instant::now()` and `SystemTime::now()`."
                .to_string()
        } else {
            format!(
                "{hits} function(s) mix monotonic and wall-clock sources and deserve temporal-integrity review."
            )
        },
        evidence,
    }
}

fn build_short_write_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_short_write_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "SHORT-WRITE",
        title: "Partial-write / Interrupted handling audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No single-call `.write(...)` sites lacking an obvious retry or `write_all` handling path were observed."
                .to_string()
        } else {
            format!(
                "{hits} function(s) call `.write(...)` without an obvious `write_all` or `Interrupted` handling path."
            )
        },
        evidence,
    }
}

fn build_async_recursion_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (unbounded_hits, bounded_hits, evidence) =
        collect_async_recursion_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    let (status, detail) = if unbounded_hits > 0 {
        (
            StructuralCheckStatus::Elevated,
            format!(
                "{unbounded_hits} async-recursive function(s) were observed without an obvious depth-limit signal."
            ),
        )
    } else if bounded_hits > 0 {
        (
            StructuralCheckStatus::Indeterminate,
            format!(
                "{bounded_hits} async-recursive function(s) were observed with a possible depth/base-case signal, but the limit is not mechanically proven."
            ),
        )
    } else {
        (
            StructuralCheckStatus::Clear,
            "No async-recursive function attributes were observed.".to_string(),
        )
    };

    AdvancedStructuralCheck {
        id: "ASYNC-RECUR",
        title: "Async recursion depth-bound audit",
        status,
        detail,
        evidence,
    }
}

fn build_unbounded_channel_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_patterns(
        documents,
        UNBOUNDED_CHANNEL_PATTERNS,
        MAX_EVIDENCE_PER_SIGNAL,
    );
    AdvancedStructuralCheck {
        id: "CHAN-UNB",
        title: "Unbounded async command-queue audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No `mpsc::unbounded_channel` sites were observed.".to_string()
        } else {
            format!(
                "{} `mpsc::unbounded_channel` site(s) were observed in the scanned crate.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn build_zero_copy_check(functions: &[FunctionSummary]) -> AdvancedStructuralCheck {
    let (hits, evidence) = collect_zero_copy_violation_evidence(functions, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "ZERO-COPY",
        title: "Copy-on-read / zero-copy provenance audit",
        status: if hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if hits == 0 {
            "No read-buffer copy-on-read motifs were observed.".to_string()
        } else {
            format!(
                "{hits} function(s) copy buffers with `.to_vec()` or `.clone()` on apparent read paths."
            )
        },
        evidence,
    }
}

fn build_version_drift_check(documents: &[SourceDocument]) -> AdvancedStructuralCheck {
    let scan = scan_dependency_version_drift(documents, MAX_EVIDENCE_PER_SIGNAL);
    AdvancedStructuralCheck {
        id: "CARGO-VERS",
        title: "Dependency version drift / reproducibility audit",
        status: if scan.total_hits == 0 {
            StructuralCheckStatus::Clear
        } else {
            StructuralCheckStatus::Elevated
        },
        detail: if scan.total_hits == 0 {
            "No wildcard or open-ended dependency version requirements were observed.".to_string()
        } else {
            format!(
                "{} dependency requirement(s) look wildcard or open-ended and deserve reproducibility review.",
                scan.total_hits
            )
        },
        evidence: scan.evidence,
    }
}

fn append_evidence(target: &mut Vec<ScanEvidence>, source: Vec<ScanEvidence>, limit: usize) {
    for item in source.into_iter() {
        if target.len() >= limit {
            break;
        }
        let already_present = target.iter().any(|existing| {
            existing.path == item.path
                && existing.line_number == item.line_number
                && existing.pattern == item.pattern
                && existing.snippet == item.snippet
        });
        if !already_present {
            target.push(item);
        }
    }
}

fn collect_direct_recursion_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        let signature_needle = format!("fn {}", function.lowered_name);

        for (offset, line) in function.body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with('*') {
                continue;
            }
            let lowered = trimmed.to_ascii_lowercase();
            if function_calls_name(&lowered, &function.lowered_name)
                && !lowered.contains(&signature_needle)
            {
                total_hits += 1;
                if evidence.len() < limit {
                    evidence.push(ScanEvidence {
                        path: function.path.clone(),
                        line_number: function.start_line + offset,
                        pattern: "direct recursion",
                        snippet: trimmed.to_string(),
                    });
                }
            }
        }
    }

    (total_hits, evidence)
}

fn collect_indirect_recursion_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut functions_by_path: BTreeMap<PathBuf, Vec<&FunctionSummary>> = BTreeMap::new();
    for function in functions {
        functions_by_path
            .entry(function.path.clone())
            .or_default()
            .push(function);
    }

    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for local_functions in functions_by_path.values() {
        for (idx, function) in local_functions.iter().enumerate() {
            for target in local_functions.iter().skip(idx + 1) {
                if function_calls_name(&function.lowered_body, &target.lowered_name)
                    && function_calls_name(&target.lowered_body, &function.lowered_name)
                {
                    hits += 1;
                    if evidence.len() < limit {
                        evidence.push(ScanEvidence {
                            path: function.path.clone(),
                            line_number: function.start_line,
                            pattern: "local indirect recursion cycle",
                            snippet: format!(
                                "function `{}` appears mutually recursive with `{}`",
                                function.name, target.name
                            ),
                        });
                    }
                }
            }
        }
    }

    (hits, evidence)
}

fn collect_complexity_hotspots(
    functions: &[FunctionSummary],
    limit: usize,
) -> Vec<CriticalityHotspot> {
    let mut hotspots = build_criticality_hotspots(functions);
    hotspots.sort_by(|a, b| {
        b.estimated_complexity
            .cmp(&a.estimated_complexity)
            .then_with(|| b.risk_score.cmp(&a.risk_score))
    });
    hotspots.into_iter().take(limit).collect()
}

fn collect_long_function_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> Vec<ScanEvidence> {
    let mut long_functions = functions
        .iter()
        .filter(|function| function.line_count > 60)
        .collect::<Vec<_>>();
    long_functions.sort_by(|a, b| b.line_count.cmp(&a.line_count));

    long_functions
        .into_iter()
        .take(limit)
        .map(|function| ScanEvidence {
            path: function.path.clone(),
            line_number: function.start_line,
            pattern: "function length > 60 lines",
            snippet: format!(
                "function `{}` spans {} lines",
                function.name, function.line_count
            ),
        })
        .collect()
}

fn collect_unbounded_iterator_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !function_contains_code_pattern(function, ITERATOR_TERMINAL_PATTERNS)
            || function_contains_code_pattern(function, ITERATOR_BOUND_PATTERNS)
            || (!function_contains_code_pattern(function, OPEN_ENDED_ITERATOR_PATTERNS)
                && !body_contains_any(&function.lowered_signature, OPEN_ENDED_ITERATOR_PATTERNS))
        {
            continue;
        }
        if let Some(item) = first_matching_line(function, ITERATOR_TERMINAL_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn collect_interrupt_safety_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !body_contains_any(&function.lowered_attributes, INTERRUPT_ATTRIBUTE_PATTERNS) {
            continue;
        }
        if let Some(item) = first_matching_line(function, ISR_FORBIDDEN_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn collect_pending_without_waker_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        let looks_like_manual_poll = function.name == "poll"
            && (function.lowered_signature.contains("-> poll<")
                || function.lowered_signature.contains("-> core::task::poll<")
                || function.lowered_signature.contains("-> std::task::poll<"));
        if !looks_like_manual_poll
            || !function_contains_code_pattern(function, MANUAL_POLL_PENDING_PATTERNS)
            || function_contains_code_pattern(function, WAKE_PATTERNS)
        {
            continue;
        }

        if let Some(item) = first_matching_line(function, MANUAL_POLL_PENDING_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn collect_panic_in_drop_evidence(
    documents: &[SourceDocument],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for document in documents.iter().filter(|document| {
        document
            .relative_path
            .extension()
            .and_then(|ext| ext.to_str())
            == Some("rs")
    }) {
        let lines = document.risk_contents.lines().collect::<Vec<_>>();
        let mut next_idx = 0usize;
        for idx in 0..lines.len() {
            if idx < next_idx {
                continue;
            }
            let lowered = lines[idx].trim().to_ascii_lowercase();
            if !lowered.contains("impl drop for ") {
                continue;
            }

            let Some((block_start, block_end)) = extract_braced_block(&lines, idx) else {
                continue;
            };

            for (line_idx, line) in lines
                .iter()
                .enumerate()
                .skip(block_start)
                .take(block_end + 1)
            {
                let lowered_line = line.trim().to_ascii_lowercase();
                for &pattern in PANIC_PATTERNS {
                    if lowered_line.contains(pattern) {
                        hits += 1;
                        if evidence.len() < limit {
                            evidence.push(ScanEvidence {
                                path: document.relative_path.clone(),
                                line_number: line_idx + 1,
                                pattern,
                                snippet: line.trim().to_string(),
                            });
                        }
                    }
                }
            }

            next_idx = block_end + 1;
        }
    }

    (hits, evidence)
}

fn collect_relaxed_atomic_state_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !function_contains_code_pattern(function, RELAXED_ORDERING_PATTERNS)
            || !function_contains_code_pattern(function, CRITICAL_STATE_PATTERNS)
        {
            continue;
        }
        if let Some(item) = first_matching_line(function, RELAXED_ORDERING_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn collect_mixed_clock_source_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !(function_contains_code_pattern(function, &["instant::now("])
            && function_contains_code_pattern(function, &["systemtime::now("]))
        {
            continue;
        }
        hits += 1;
        if evidence.len() < limit {
            evidence.push(ScanEvidence {
                path: function.path.clone(),
                line_number: function.start_line,
                pattern: "instant::now() + systemtime::now()",
                snippet: format!(
                    "function `{}` mixes monotonic and wall-clock time sources",
                    function.name
                ),
            });
        }
    }

    (hits, evidence)
}

fn collect_short_write_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !function_contains_code_pattern(function, WRITE_CALL_PATTERNS)
            || function_contains_code_pattern(function, &["write_all("])
            || function_contains_code_pattern(function, WRITE_HANDLING_PATTERNS)
        {
            continue;
        }
        if let Some(item) = first_matching_line(function, WRITE_CALL_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn collect_async_recursion_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, usize, Vec<ScanEvidence>) {
    let mut unbounded_hits = 0usize;
    let mut bounded_hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        if !body_contains_any(&function.lowered_attributes, ASYNC_RECURSION_PATTERNS) {
            continue;
        }

        let has_depth_signal = function_contains_code_pattern(function, DEPTH_BOUND_PATTERNS)
            || body_contains_any(&function.lowered_signature, DEPTH_BOUND_PATTERNS)
            || body_contains_any(&function.lowered_attributes, DEPTH_BOUND_PATTERNS);

        if has_depth_signal {
            bounded_hits += 1;
        } else {
            unbounded_hits += 1;
        }

        if evidence.len() < limit {
            evidence.push(ScanEvidence {
                path: function.path.clone(),
                line_number: function.start_line,
                pattern: "#[async_recursion]",
                snippet: if has_depth_signal {
                    format!(
                        "function `{}` uses async recursion with a possible depth/base-case signal",
                        function.name
                    )
                } else {
                    format!(
                        "function `{}` uses async recursion without an obvious depth-limit signal",
                        function.name
                    )
                },
            });
        }
    }

    (unbounded_hits, bounded_hits, evidence)
}

fn collect_zero_copy_violation_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for function in functions {
        let has_copy_input_context =
            body_contains_any(&function.lowered_signature, READ_BUFFER_SIGNATURE_PATTERNS);
        if !has_copy_input_context
            || !function_contains_code_pattern(function, COPY_ON_READ_PATTERNS)
        {
            continue;
        }
        if let Some(item) = first_matching_line(function, COPY_ON_READ_PATTERNS) {
            hits += 1;
            if evidence.len() < limit {
                evidence.push(item);
            }
        }
    }

    (hits, evidence)
}

fn build_criticality_hotspots(functions: &[FunctionSummary]) -> Vec<CriticalityHotspot> {
    let mut hotspots = Vec::new();

    for function in functions {
        let signals = collect_hotspot_signals(function);
        let risk_score = compute_hotspot_risk_score(function, &signals);

        if risk_score >= 12 {
            hotspots.push(CriticalityHotspot {
                path: function.path.clone(),
                function_name: function.name.clone(),
                start_line: function.start_line,
                estimated_complexity: function.estimated_complexity,
                risk_score,
                signals,
            });
        }
    }

    hotspots.sort_by(|a, b| {
        b.risk_score
            .cmp(&a.risk_score)
            .then_with(|| b.estimated_complexity.cmp(&a.estimated_complexity))
    });
    hotspots.truncate(8);
    hotspots
}

fn collect_hotspot_signals(function: &FunctionSummary) -> Vec<&'static str> {
    let mut signals = Vec::new();

    push_basic_hotspot_signals(function, &mut signals);
    push_behavioral_hotspot_signals(function, &mut signals);
    if has_hotspot_unbounded_iterator(function) {
        signals.push("iter-unbounded");
    }
    if has_hotspot_relaxed_atomic_signal(function) {
        signals.push("relaxed-atomic");
    }
    if has_hotspot_mixed_clock_signal(function) {
        signals.push("mixed-clocks");
    }
    if function_contains_code_pattern(function, UNBOUNDED_CHANNEL_PATTERNS) {
        signals.push("unbounded-channel");
    }
    if has_hotspot_pending_without_wake_signal(function) {
        signals.push("pending-no-wake");
    }
    if has_hotspot_copy_on_read_signal(function) {
        signals.push("copy-on-read");
    }

    signals
}

fn push_basic_hotspot_signals(function: &FunctionSummary, signals: &mut Vec<&'static str>) {
    if function.estimated_complexity > 15 {
        signals.push("complexity>15");
    }
    if function.line_count > 60 {
        signals.push("long-function");
    }
    if function_contains_code_pattern(function, ASSERT_PATTERNS) && function.assertion_count < 2 {
        signals.push("low-assert-density");
    }
    if function_contains_code_pattern(function, UNWRAP_PATTERNS) {
        signals.push("unwrap");
    }
    if function_contains_code_pattern(function, UNSAFE_PATTERNS) {
        signals.push("unsafe");
    }
}

fn push_behavioral_hotspot_signals(function: &FunctionSummary, signals: &mut Vec<&'static str>) {
    if function_contains_code_pattern(function, HARD_CODED_WAIT_PATTERNS) {
        signals.push("hard-coded-wait");
    }
    if function_contains_code_pattern(function, ASYNC_LOCK_PATTERNS) {
        signals.push("async-lock");
    }
    if function_contains_code_pattern(function, INTERIOR_MUTABILITY_PATTERNS) {
        signals.push("interior-mutability");
    }
    if function_contains_code_pattern(function, COMMAND_BUFFER_PATTERNS) {
        signals.push("command-buffer");
    }
}

fn has_hotspot_unbounded_iterator(function: &FunctionSummary) -> bool {
    function_contains_code_pattern(function, ITERATOR_TERMINAL_PATTERNS)
        && !function_contains_code_pattern(function, ITERATOR_BOUND_PATTERNS)
        && (function_contains_code_pattern(function, OPEN_ENDED_ITERATOR_PATTERNS)
            || body_contains_any(&function.lowered_signature, OPEN_ENDED_ITERATOR_PATTERNS))
}

fn has_hotspot_relaxed_atomic_signal(function: &FunctionSummary) -> bool {
    function_contains_code_pattern(function, RELAXED_ORDERING_PATTERNS)
        && function_contains_code_pattern(function, CRITICAL_STATE_PATTERNS)
}

fn has_hotspot_mixed_clock_signal(function: &FunctionSummary) -> bool {
    function_contains_code_pattern(function, &["instant::now("])
        && function_contains_code_pattern(function, &["systemtime::now("])
}

fn has_hotspot_pending_without_wake_signal(function: &FunctionSummary) -> bool {
    function.name == "poll"
        && (function.lowered_signature.contains("-> poll<")
            || function.lowered_signature.contains("-> core::task::poll<")
            || function.lowered_signature.contains("-> std::task::poll<"))
        && function_contains_code_pattern(function, MANUAL_POLL_PENDING_PATTERNS)
        && !function_contains_code_pattern(function, WAKE_PATTERNS)
}

fn has_hotspot_copy_on_read_signal(function: &FunctionSummary) -> bool {
    function_contains_code_pattern(function, COPY_ON_READ_PATTERNS)
        && body_contains_any(&function.lowered_signature, READ_BUFFER_SIGNATURE_PATTERNS)
}

fn compute_hotspot_risk_score(function: &FunctionSummary, signals: &[&'static str]) -> usize {
    function.estimated_complexity
        + signals.len() * 3
        + usize::from(function.line_count > 60) * 5
        + usize::from(function_contains_code_pattern(function, UNSAFE_PATTERNS)) * 6
        + usize::from(function_contains_code_pattern(function, UNWRAP_PATTERNS)) * 3
}

fn collect_low_assertion_evidence(
    functions: &[FunctionSummary],
    limit: usize,
) -> Vec<ScanEvidence> {
    let mut sparse_functions = functions
        .iter()
        .map(|function| (function.assertion_count, function))
        .filter(|(assertions, _)| *assertions < 2)
        .collect::<Vec<_>>();
    sparse_functions.sort_by(|(left_asserts, left_fn), (right_asserts, right_fn)| {
        left_asserts
            .cmp(right_asserts)
            .then_with(|| right_fn.line_count.cmp(&left_fn.line_count))
    });

    sparse_functions
        .into_iter()
        .take(limit)
        .map(|(assertions, function)| ScanEvidence {
            path: function.path.clone(),
            line_number: function.start_line,
            pattern: "assertion density < 2 per function",
            snippet: format!(
                "function `{}` has {} assertion site(s) across {} lines",
                function.name, assertions, function.line_count
            ),
        })
        .collect()
}

fn estimate_cyclomatic_complexity_lowered(lowered: &str) -> usize {
    let mut complexity = 1usize;
    let decision_tokens = [
        "if ",
        "else if ",
        "match ",
        "while ",
        "while let ",
        "for ",
        "loop ",
        "&&",
        "||",
    ];

    for token in decision_tokens.iter().copied() {
        complexity += lowered.match_indices(token).count();
    }

    complexity
}

fn body_contains_any(lowered_body: &str, patterns: &[&'static str]) -> bool {
    patterns
        .iter()
        .any(|pattern| lowered_body.contains(pattern))
}

fn is_code_like_scan_line(trimmed: &str) -> bool {
    !(trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("///")
        || trimmed.starts_with('*')
        || trimmed.starts_with('"')
        || trimmed.starts_with("b\"")
        || trimmed.starts_with("r\"")
        || trimmed.starts_with("r#\""))
}

fn function_contains_code_pattern(function: &FunctionSummary, patterns: &[&'static str]) -> bool {
    function.body.lines().any(|line| {
        let trimmed = line.trim();
        if !is_code_like_scan_line(trimmed) {
            return false;
        }
        let lowered = strip_rust_comments_and_strings(trimmed).to_ascii_lowercase();
        patterns.iter().any(|pattern| lowered.contains(pattern))
    })
}

fn function_calls_name(lowered_body: &str, target_name_lowered: &str) -> bool {
    let needle = format!("{target_name_lowered}(");
    for (absolute, _) in lowered_body.match_indices(&needle) {
        let prefix = &lowered_body[..absolute];

        if !prefix.ends_with("fn ")
            && (prefix.ends_with("self.")
                || prefix.ends_with("self::")
                || prefix.ends_with("super::")
                || prefix.ends_with("crate::")
                || prefix.chars().last().is_none_or(|ch| {
                    !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.' && ch != ':'
                }))
        {
            return true;
        }
    }

    false
}

fn count_assertions_in_text(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with('*')
        })
        .map(|line| {
            let lowered = line.to_ascii_lowercase();
            ASSERT_PATTERNS
                .iter()
                .map(|pattern| lowered.match_indices(pattern).count())
                .sum::<usize>()
        })
        .sum()
}

fn first_matching_line(
    function: &FunctionSummary,
    patterns: &[&'static str],
) -> Option<ScanEvidence> {
    for (offset, line) in function.body.lines().enumerate() {
        let trimmed = line.trim();
        if !is_code_like_scan_line(trimmed) {
            continue;
        }
        let lowered = strip_rust_comments_and_strings(trimmed).to_ascii_lowercase();
        for &pattern in patterns {
            if lowered.contains(pattern) {
                return Some(ScanEvidence {
                    path: function.path.clone(),
                    line_number: function.start_line + offset,
                    pattern,
                    snippet: trimmed.to_string(),
                });
            }
        }
    }

    None
}

fn extract_braced_block(lines: &[&str], start_idx: usize) -> Option<(usize, usize)> {
    let mut brace_balance = 0isize;
    let mut seen_open_brace = false;
    let mut block_start = None;

    for (idx, line) in lines.iter().enumerate().skip(start_idx) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    brace_balance += 1;
                    if !seen_open_brace {
                        seen_open_brace = true;
                        block_start = Some(idx);
                    }
                }
                '}' => {
                    brace_balance -= 1;
                }
                _other => {}
            }
        }

        if seen_open_brace && brace_balance == 0 {
            return block_start.map(|start| (start, idx));
        }
    }

    None
}

fn extract_function_summaries(documents: &[SourceDocument]) -> Vec<FunctionSummary> {
    let mut functions = Vec::new();

    for document in documents.iter().filter(|document| {
        document
            .relative_path
            .extension()
            .and_then(|ext| ext.to_str())
            == Some("rs")
    }) {
        let lines = document.risk_contents.lines().collect::<Vec<_>>();
        let test_section_start = lines
            .iter()
            .position(|line| line.trim_start().starts_with("#[cfg(test)]"));
        let mut next_idx = 0usize;
        for idx in 0..lines.len() {
            if idx < next_idx {
                continue;
            }
            if test_section_start.is_some_and(|start| idx >= start) {
                break;
            }
            if let Some((function, end_idx)) =
                try_extract_function(&document.relative_path, &lines, idx)
            {
                functions.push(function);
                next_idx = end_idx + 1;
            }
        }
    }

    functions
}

fn try_extract_function(
    path: &Path,
    lines: &[&str],
    start_idx: usize,
) -> Option<(FunctionSummary, usize)> {
    let attribute_start_idx = rewind_attribute_start(lines, start_idx);
    let lowered_attributes = lowered_attribute_block(lines, attribute_start_idx, start_idx);
    let (signature, signature_end_idx) = collect_function_signature(lines, start_idx)?;
    let name = extract_function_name_from_signature(&signature)?;
    let end_idx = repaired_function_end_idx(
        lines,
        start_idx,
        find_function_end_idx(lines, signature_end_idx)?,
    );
    let summary = build_function_summary(
        path,
        name,
        signature,
        lowered_attributes,
        start_idx,
        end_idx,
        lines,
    );

    Some((summary, end_idx))
}

fn repaired_function_end_idx(lines: &[&str], start_idx: usize, end_idx: usize) -> usize {
    let mut previous_nonempty_idx = start_idx;

    for idx in (start_idx + 1)..=end_idx {
        let trimmed = lines[idx].trim();
        if trimmed.is_empty() {
            continue;
        }
        if appears_to_start_function_signature(trimmed) && lines[idx].starts_with(trimmed) {
            let previous = lines[previous_nonempty_idx].trim();
            if previous == "}" {
                return previous_nonempty_idx;
            }
        }
        previous_nonempty_idx = idx;
    }

    end_idx
}

fn appears_to_start_function_signature(line: &str) -> bool {
    if line.is_empty()
        || line.starts_with('#')
        || line.starts_with("//")
        || line.starts_with("///")
        || line.starts_with("macro_rules!")
    {
        return false;
    }

    line.contains("fn ")
        && !line.starts_with("if ")
        && !line.starts_with("while ")
        && !line.starts_with("for ")
        && !line.starts_with("match ")
}

fn extract_function_name_from_signature(signature: &str) -> Option<String> {
    let fn_pos = signature.find("fn ")?;
    let rest = &signature[fn_pos + 3..];
    let name = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn update_brace_balance_from_code_line(
    line: &str,
    brace_balance: &mut isize,
    seen_open_brace: &mut bool,
) {
    let sanitized = strip_rust_comments_and_strings(line)
        .replace("'{'", "   ")
        .replace("'}'", "   ");
    for ch in sanitized.chars() {
        match ch {
            '{' => {
                *brace_balance += 1;
                *seen_open_brace = true;
            }
            '}' => *brace_balance -= 1,
            _other => {}
        }
    }
}

fn rewind_attribute_start(lines: &[&str], start_idx: usize) -> usize {
    let mut attribute_start_idx = start_idx;
    for previous_idx in (0..attribute_start_idx).rev() {
        let previous = lines[previous_idx].trim();
        if previous.starts_with('#') {
            attribute_start_idx = previous_idx;
        } else {
            break;
        }
    }
    attribute_start_idx
}

fn lowered_attribute_block(lines: &[&str], attribute_start_idx: usize, start_idx: usize) -> String {
    lines[attribute_start_idx..start_idx]
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase()
}

fn collect_function_signature(lines: &[&str], start_idx: usize) -> Option<(String, usize)> {
    let mut signature = String::new();

    for (idx, line) in lines.iter().enumerate().skip(start_idx) {
        let trimmed = line.trim();
        if idx == start_idx && !appears_to_start_function_signature(trimmed) {
            return None;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with('*') {
            return None;
        }

        if !signature.is_empty() {
            signature.push(' ');
        }
        signature.push_str(trimmed);

        if signature.contains(';') && !signature.contains('{') {
            return None;
        }
        if signature.contains('{') {
            return Some((signature, idx));
        }
    }

    None
}

fn find_function_end_idx(lines: &[&str], signature_end_idx: usize) -> Option<usize> {
    let mut brace_balance = 0isize;
    let mut seen_open_brace = false;
    let mut last_standalone_closing_brace_idx = None;

    for (idx, line) in lines.iter().enumerate().skip(signature_end_idx) {
        update_brace_balance_from_code_line(line, &mut brace_balance, &mut seen_open_brace);
        if line.trim() == "}" {
            last_standalone_closing_brace_idx = Some(idx);
        }
        if seen_open_brace && brace_balance == 0 {
            return Some(idx);
        }
        let trimmed = line.trim();
        if seen_open_brace
            && idx > signature_end_idx
            && line.starts_with(trimmed)
            && appears_to_start_function_signature(trimmed)
            && last_standalone_closing_brace_idx.is_some()
        {
            return last_standalone_closing_brace_idx;
        }
    }

    None
}

fn build_function_summary(
    path: &Path,
    name: String,
    signature: String,
    lowered_attributes: String,
    start_idx: usize,
    end_idx: usize,
    lines: &[&str],
) -> FunctionSummary {
    let body = lines[start_idx..=end_idx].join("\n");
    let lowered_body = body.to_ascii_lowercase();

    FunctionSummary {
        path: path.to_path_buf(),
        lowered_name: name.to_ascii_lowercase(),
        name,
        lowered_signature: signature.to_ascii_lowercase(),
        lowered_attributes,
        start_line: start_idx + 1,
        line_count: end_idx - start_idx + 1,
        estimated_complexity: estimate_cyclomatic_complexity_lowered(&lowered_body),
        assertion_count: count_assertions_in_text(&body),
        body,
        lowered_body,
    }
}

fn scan_manifest(path: &Path) -> ManifestMetadata {
    let Ok(contents) = fs::read_to_string(path) else {
        return ManifestMetadata::default();
    };

    let mut metadata = ManifestMetadata::default();
    let mut section = ManifestSection::None;
    let mut direct_dependencies = BTreeSet::new();
    let mut build_dependencies = BTreeSet::new();
    let mut dev_dependencies = BTreeSet::new();

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = update_manifest_section(
                line,
                &mut direct_dependencies,
                &mut build_dependencies,
                &mut dev_dependencies,
            );
            continue;
        }

        apply_manifest_value_line(
            &mut metadata,
            section,
            raw_line,
            line,
            &mut direct_dependencies,
            &mut build_dependencies,
            &mut dev_dependencies,
        );
    }

    set_manifest_dependency_counts(
        &mut metadata,
        &direct_dependencies,
        &build_dependencies,
        &dev_dependencies,
    );
    metadata
}

fn update_manifest_section(
    line: &str,
    direct_dependencies: &mut BTreeSet<String>,
    build_dependencies: &mut BTreeSet<String>,
    dev_dependencies: &mut BTreeSet<String>,
) -> ManifestSection {
    let section = classify_manifest_section(line);
    insert_dependency_from_section_header(
        direct_dependencies,
        line,
        ManifestSection::Dependencies,
        "dependencies",
        section,
    );
    insert_dependency_from_section_header(
        build_dependencies,
        line,
        ManifestSection::BuildDependencies,
        "build-dependencies",
        section,
    );
    insert_dependency_from_section_header(
        dev_dependencies,
        line,
        ManifestSection::DevDependencies,
        "dev-dependencies",
        section,
    );
    section
}

fn apply_manifest_value_line(
    metadata: &mut ManifestMetadata,
    section: ManifestSection,
    raw_line: &str,
    line: &str,
    direct_dependencies: &mut BTreeSet<String>,
    build_dependencies: &mut BTreeSet<String>,
    dev_dependencies: &mut BTreeSet<String>,
) {
    match section {
        ManifestSection::Package => apply_package_manifest_line(metadata, line),
        ManifestSection::Lib => apply_lib_manifest_line(metadata, line),
        ManifestSection::Dependencies => {
            insert_dependency_from_key(direct_dependencies, raw_line, line)
        }
        ManifestSection::BuildDependencies => {
            insert_dependency_from_key(build_dependencies, raw_line, line)
        }
        ManifestSection::DevDependencies => {
            insert_dependency_from_key(dev_dependencies, raw_line, line)
        }
        ManifestSection::None => {}
    }
}

fn set_manifest_dependency_counts(
    metadata: &mut ManifestMetadata,
    direct_dependencies: &BTreeSet<String>,
    build_dependencies: &BTreeSet<String>,
    dev_dependencies: &BTreeSet<String>,
) {
    metadata.direct_dependencies = direct_dependencies.len();
    metadata.build_dependencies = build_dependencies.len();
    metadata.dev_dependencies = dev_dependencies.len();
}

fn insert_dependency_from_section_header(
    target: &mut BTreeSet<String>,
    line: &str,
    expected_section: ManifestSection,
    section_name: &str,
    section: ManifestSection,
) {
    if section == expected_section {
        if let Some(name) = dependency_name_from_section(line, section_name) {
            target.insert(name);
        }
    }
}

fn apply_package_manifest_line(metadata: &mut ManifestMetadata, line: &str) {
    if apply_package_scalar_field(metadata, line) {
        return;
    }
    apply_build_manifest_field(metadata, line);
}

fn apply_package_scalar_field(metadata: &mut ManifestMetadata, line: &str) -> bool {
    let Some((key, value)) = parse_package_scalar_field(line) else {
        return false;
    };
    assign_package_scalar(metadata, key, value);
    true
}

fn assign_package_scalar(metadata: &mut ManifestMetadata, key: &'static str, value: String) {
    match key {
        "name" => metadata.crate_name = Some(value),
        "version" => metadata.crate_version = Some(value),
        "edition" => metadata.edition = Some(value),
        "license" => metadata.license = Some(value),
        "rust-version" => metadata.rust_version = Some(value),
        "repository" => metadata.repository = Some(value),
        "homepage" => metadata.homepage = Some(value),
        "documentation" => metadata.documentation = Some(value),
        "readme" => metadata.readme = Some(value),
        _other => {}
    }
}

fn apply_build_manifest_field(metadata: &mut ManifestMetadata, line: &str) {
    match parse_build_manifest_value(line) {
        Some(BuildManifestValue::Disabled) => metadata.build_script = None,
        Some(BuildManifestValue::DefaultScript) => {
            metadata.build_script = Some("build.rs".to_string());
        }
        Some(BuildManifestValue::Path(path)) => metadata.build_script = Some(path),
        None => {}
    }
}

fn parse_build_manifest_value(line: &str) -> Option<BuildManifestValue> {
    match parse_manifest_bool(line, "build") {
        Some(false) => Some(BuildManifestValue::Disabled),
        Some(true) => Some(BuildManifestValue::DefaultScript),
        None => parse_manifest_value(line, "build").map(BuildManifestValue::Path),
    }
}

enum BuildManifestValue {
    Disabled,
    DefaultScript,
    Path(String),
}

fn parse_package_scalar_field(line: &str) -> Option<(&'static str, String)> {
    for key in [
        "name",
        "version",
        "edition",
        "license",
        "rust-version",
        "repository",
        "homepage",
        "documentation",
        "readme",
    ] {
        if let Some(value) = parse_manifest_value(line, key) {
            return Some((key, value));
        }
    }

    None
}

fn apply_lib_manifest_line(metadata: &mut ManifestMetadata, line: &str) {
    if parse_manifest_bool(line, "proc-macro") == Some(true) {
        metadata.proc_macro = true;
    }
}

fn insert_dependency_from_key(target: &mut BTreeSet<String>, raw_line: &str, line: &str) {
    if let Some(key) = parse_dependency_key(raw_line, line) {
        target.insert(key);
    }
}

fn scan_patterns(
    documents: &[SourceDocument],
    patterns: &[&'static str],
    max_evidence: usize,
) -> PatternScan {
    scan_patterns_with_selector(documents, patterns, max_evidence, ScanContentMode::Risk)
}

fn scan_patterns_analysis(
    documents: &[SourceDocument],
    patterns: &[&'static str],
    max_evidence: usize,
) -> PatternScan {
    scan_patterns_with_selector(documents, patterns, max_evidence, ScanContentMode::Analysis)
}

fn scan_patterns_filtered<F>(
    documents: &[SourceDocument],
    patterns: &[&'static str],
    max_evidence: usize,
    mode: ScanContentMode,
    include_path: F,
) -> PatternScan
where
    F: Fn(&Path) -> bool,
{
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        if !include_path(&document.relative_path) {
            continue;
        }

        let source_lines = document.contents.lines();
        let scan_lines = match mode {
            ScanContentMode::Analysis => document.analysis_contents.lines(),
            ScanContentMode::Risk => document.risk_contents.lines(),
        };

        for (idx, (source_line, scan_line)) in source_lines.zip(scan_lines).enumerate() {
            let lowered = scan_line.to_ascii_lowercase();
            for &pattern in patterns {
                if lowered.contains(pattern) {
                    total_hits += 1;
                    matched_patterns.insert(pattern);
                    if evidence.len() < max_evidence {
                        evidence.push(ScanEvidence {
                            path: document.relative_path.clone(),
                            line_number: idx + 1,
                            pattern,
                            snippet: source_line.trim().to_string(),
                        });
                    }
                }
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

#[derive(Clone, Copy)]
enum ScanContentMode {
    Analysis,
    Risk,
}

fn scan_patterns_with_selector(
    documents: &[SourceDocument],
    patterns: &[&'static str],
    max_evidence: usize,
    mode: ScanContentMode,
) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        let source_lines = document.contents.lines();
        let scan_lines = match mode {
            ScanContentMode::Analysis => document.analysis_contents.lines(),
            ScanContentMode::Risk => document.risk_contents.lines(),
        };

        for (idx, (source_line, scan_line)) in source_lines.zip(scan_lines).enumerate() {
            let lowered = scan_line.to_ascii_lowercase();
            for &pattern in patterns {
                if lowered.contains(pattern) {
                    total_hits += 1;
                    matched_patterns.insert(pattern);
                    if evidence.len() < max_evidence {
                        evidence.push(ScanEvidence {
                            path: document.relative_path.clone(),
                            line_number: idx + 1,
                            pattern,
                            snippet: source_line.trim().to_string(),
                        });
                    }
                }
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn line_contains_bounded_while_condition(line: &str) -> bool {
    let lowered = line.trim().to_ascii_lowercase();
    is_bounded_iterator_while(&lowered) || is_bounded_numeric_while(&lowered)
}

fn is_bounded_iterator_while(lowered: &str) -> bool {
    body_contains_any(
        lowered,
        &[
            "while let some(",
            "while let some (",
            "while let ok(",
            "while let ok (",
        ],
    ) && body_contains_any(lowered, &[".next()", ".pop()", ".find(", ".peek()"])
}

fn is_bounded_numeric_while(lowered: &str) -> bool {
    lowered.contains("while ")
        && body_contains_any(lowered, &[".len()", " > 0", " != 0", " < ", " <= "])
}

fn collect_unbounded_loop_evidence(
    documents: &[SourceDocument],
    max_evidence: usize,
) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        for (idx, (source_line, risk_line)) in document
            .contents
            .lines()
            .zip(document.risk_contents.lines())
            .enumerate()
        {
            let lowered = risk_line.trim().to_ascii_lowercase();

            if lowered.contains("loop {") || lowered.contains("loop{") {
                total_hits += 1;
                matched_patterns.insert("loop");
                if evidence.len() < max_evidence {
                    evidence.push(ScanEvidence {
                        path: document.relative_path.clone(),
                        line_number: idx + 1,
                        pattern: "loop",
                        snippet: source_line.trim().to_string(),
                    });
                }
                continue;
            }

            if (lowered.contains("while let ") || lowered.contains("while "))
                && !line_contains_bounded_while_condition(&lowered)
            {
                total_hits += 1;
                matched_patterns.insert("while");
                if evidence.len() < max_evidence {
                    evidence.push(ScanEvidence {
                        path: document.relative_path.clone(),
                        line_number: idx + 1,
                        pattern: "while",
                        snippet: source_line.trim().to_string(),
                    });
                }
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn collect_ambiguous_for_loop_evidence(
    documents: &[SourceDocument],
    max_evidence: usize,
) -> (usize, Vec<ScanEvidence>) {
    let mut hits = 0usize;
    let mut evidence = Vec::new();

    for document in documents.iter().filter(|document| {
        document
            .relative_path
            .extension()
            .and_then(|ext| ext.to_str())
            == Some("rs")
    }) {
        for (idx, source_line) in document.contents.lines().enumerate() {
            let trimmed = source_line.trim();
            if !trimmed.starts_with("for ") {
                continue;
            }
            let loop_signature = collect_for_loop_signature(&document.contents, idx);
            if line_contains_bounded_for_loop(&loop_signature) {
                continue;
            }

            hits += 1;
            if evidence.len() < max_evidence {
                evidence.push(ScanEvidence {
                    path: document.relative_path.clone(),
                    line_number: idx + 1,
                    pattern: "for loop with non-obvious bound",
                    snippet: trimmed.to_string(),
                });
            }
        }
    }

    (hits, evidence)
}

fn collect_for_loop_signature(contents: &str, start_idx: usize) -> String {
    let lines = contents.lines().collect::<Vec<_>>();
    let mut signature = String::new();

    for line in lines.iter().skip(start_idx).take(6) {
        if !signature.is_empty() {
            signature.push(' ');
        }
        signature.push_str(line.trim());
        if line.contains('{') {
            break;
        }
    }

    signature
}

fn line_contains_bounded_for_loop(line: &str) -> bool {
    let lowered = line.trim().to_ascii_lowercase();
    if !lowered.starts_with("for ") {
        return false;
    }

    contains_bounded_for_iter_pattern(&lowered) || contains_known_bounded_for_target(&lowered)
}

fn scan_cfg_surface(documents: &[SourceDocument], max_evidence: usize) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents
        .iter()
        .filter(|document| !has_path_component(&document.relative_path, "proofs"))
    {
        for (idx, (source_line, scan_line)) in document
            .contents
            .lines()
            .zip(document.analysis_contents.lines())
            .enumerate()
        {
            if !is_review_relevant_cfg_line(source_line.trim(), scan_line.trim()) {
                continue;
            }

            total_hits += 1;
            matched_patterns.insert("cfg");
            if evidence.len() < max_evidence {
                evidence.push(ScanEvidence {
                    path: document.relative_path.clone(),
                    line_number: idx + 1,
                    pattern: "review-relevant cfg",
                    snippet: source_line.trim().to_string(),
                });
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn is_review_relevant_cfg_line(raw_trimmed: &str, scan_trimmed: &str) -> bool {
    if !(scan_trimmed.contains("#[cfg")
        || scan_trimmed.contains("cfg!(")
        || scan_trimmed.contains("cfg_attr("))
    {
        return false;
    }
    !is_ignored_cfg_line(raw_trimmed)
}

fn is_ignored_cfg_line(raw_trimmed: &str) -> bool {
    raw_trimmed.contains("cfg(test)")
        || raw_trimmed.contains("cfg_attr(test")
        || raw_trimmed == "#[cfg(feature = \"std\")]"
        || raw_trimmed == "#[cfg(not(feature = \"std\"))]"
        || raw_trimmed.contains("cfg_attr(not(feature = \"std\"), no_std)")
        || raw_trimmed.contains("cfg_attr(not(any(feature = \"std\")), no_std)")
        || raw_trimmed.contains("cfg_attr(not(test), forbid(unsafe_code))")
        || raw_trimmed.contains("cfg_attr(not(test), deny(unsafe_code))")
}

fn contains_bounded_for_iter_pattern(lowered: &str) -> bool {
    body_contains_any(
        lowered,
        &[
            " in 0..",
            " in 1..",
            " in ..",
            ".iter(",
            ".iter()",
            ".iter_mut(",
            ".iter_mut()",
            ".enumerate(",
            ".enumerate()",
            ".chunks(",
            ".windows(",
            ".split(",
            ".split_whitespace(",
            ".lines(",
            ".bytes(",
            ".char_indices(",
            ".chars(",
            ".keys(",
            ".match_indices(",
            ".values(",
            ".values_mut(",
            ".drain(",
            ".into_iter()",
            ".into_iter(",
            "fs::read_dir(",
            " in &",
            " in [",
            " in (",
            " in vec![",
        ],
    )
}

fn contains_known_bounded_for_target(lowered: &str) -> bool {
    lowered.contains(" in ")
        && [
            "documents",
            "functions",
            "sections",
            "checks",
            "rows",
            "patterns",
            "hotspots",
            "samples",
            "files",
            "entries",
            "bundle",
            "sigma_values",
            "p_values",
        ]
        .iter()
        .any(|pattern| lowered.contains(pattern))
}

fn collect_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![PathBuf::from(".")];
    let mut pending_idx = 0usize;

    while pending_idx < pending.len() {
        let relative_dir = pending[pending_idx].clone();
        pending_idx += 1;
        let current = if relative_dir == Path::new(".") {
            root.to_path_buf()
        } else {
            root.join(&relative_dir)
        };

        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            let file_type = entry.file_type()?;

            if should_skip_scan_path(&relative_path) {
                continue;
            }

            if file_type.is_dir() {
                pending.push(relative_path);
            } else if file_type.is_file() {
                files.push(relative_path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn should_skip_scan_path(path: &Path) -> bool {
    if has_path_component(path, ".git")
        || has_path_component(path, "target")
        || has_path_component(path, DEFAULT_SCAN_OUTPUT_ROOT)
    {
        return true;
    }

    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized.starts_with("docs/generated/")
        || normalized.starts_with("paper/generated/")
        || normalized.ends_with("_scan.txt")
        || normalized.ends_with("_scan.sarif.json")
        || normalized.ends_with("_scan.intoto.json")
        || normalized.ends_with("_scan.dsse.json")
}

fn is_tooling_support_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized.starts_with("src/bin/")
        || normalized == "src/evaluation.rs"
        || normalized == "src/inject.rs"
        || normalized == "src/report.rs"
        || normalized == "src/scan.rs"
        || normalized.starts_with("examples/")
        || normalized.starts_with("proofs/")
        || normalized.starts_with("fuzz/")
        || normalized.starts_with("tests/")
}

fn load_documents(root: &Path, files: &[PathBuf]) -> Vec<SourceDocument> {
    let mut documents = Vec::new();

    for relative_path in files {
        let absolute_path = root.join(relative_path);
        let Ok(contents) = fs::read_to_string(&absolute_path) else {
            continue;
        };
        let analysis_contents = build_analysis_contents(relative_path, &contents);
        documents.push(SourceDocument {
            relative_path: relative_path.clone(),
            risk_contents: build_risk_contents(relative_path, &analysis_contents),
            analysis_contents,
            contents,
        });
    }

    documents
}

fn build_analysis_contents(path: &Path, contents: &str) -> String {
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return contents.to_string();
    }

    strip_rust_comments_and_strings(contents)
}

fn build_risk_contents(path: &Path, analysis_contents: &str) -> String {
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return analysis_contents.to_string();
    }

    if has_path_component(path, "tests")
        || has_path_component(path, "fuzz")
        || has_path_component(path, "benches")
    {
        return blank_contents_preserving_lines(analysis_contents);
    }

    strip_cfg_test_modules(analysis_contents)
}

fn blank_contents_preserving_lines(contents: &str) -> String {
    contents.lines().map(|_| "").collect::<Vec<_>>().join("\n")
}

fn strip_cfg_test_modules(contents: &str) -> String {
    let mut output = Vec::new();
    let mut in_test_section = false;

    for line in contents.lines() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            in_test_section = true;
        }

        if in_test_section {
            output.push(String::new());
        } else {
            output.push(line.to_string());
        }
    }

    output.join("\n")
}

fn strip_rust_comments_and_strings(contents: &str) -> String {
    let mut cleaned = String::with_capacity(contents.len());
    let mut chars = contents.chars().peekable();
    let mut state = StripState::Code;

    while let Some(ch) = chars.next() {
        let next = chars.peek().copied();
        let starts_char_literal = matches!(state, StripState::Code)
            && ch == '\''
            && starts_rust_char_literal(chars.clone());

        match state {
            StripState::Code => {
                state = handle_code_strip_state(
                    &mut cleaned,
                    &mut chars,
                    ch,
                    next,
                    starts_char_literal,
                );
            }
            StripState::LineComment => {
                state = handle_line_comment_strip_state(&mut cleaned, ch);
            }
            StripState::BlockComment => {
                state = handle_block_comment_strip_state(&mut cleaned, &mut chars, ch, next);
            }
            StripState::String { escaped } => {
                state = handle_string_strip_state(&mut cleaned, ch, escaped);
            }
            StripState::Char { escaped } => {
                state = handle_char_strip_state(&mut cleaned, ch, escaped);
            }
        }
    }

    cleaned
}

fn handle_code_strip_state(
    cleaned: &mut String,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ch: char,
    next: Option<char>,
    starts_char_literal: bool,
) -> StripState {
    if ch == '/' && next == Some('*') {
        cleaned.push(' ');
        cleaned.push(' ');
        chars.next();
        StripState::BlockComment
    } else if ch == '/' && next == Some('/') {
        cleaned.push(' ');
        cleaned.push(' ');
        chars.next();
        StripState::LineComment
    } else if ch == '"' {
        cleaned.push(' ');
        StripState::String { escaped: false }
    } else if starts_char_literal {
        cleaned.push(' ');
        StripState::Char { escaped: false }
    } else {
        cleaned.push(ch);
        StripState::Code
    }
}

fn handle_line_comment_strip_state(cleaned: &mut String, ch: char) -> StripState {
    if ch == '\n' {
        cleaned.push('\n');
        StripState::Code
    } else {
        cleaned.push(' ');
        StripState::LineComment
    }
}

fn handle_block_comment_strip_state(
    cleaned: &mut String,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ch: char,
    next: Option<char>,
) -> StripState {
    if ch == '*' && next == Some('/') {
        cleaned.push(' ');
        cleaned.push(' ');
        chars.next();
        StripState::Code
    } else {
        cleaned.push(if ch == '\n' { '\n' } else { ' ' });
        StripState::BlockComment
    }
}

fn handle_string_strip_state(cleaned: &mut String, ch: char, escaped: bool) -> StripState {
    if ch == '\n' {
        cleaned.push('\n');
        StripState::String { escaped: false }
    } else if escaped {
        cleaned.push(' ');
        StripState::String { escaped: false }
    } else if ch == '\\' {
        cleaned.push(' ');
        StripState::String { escaped: true }
    } else if ch == '"' {
        cleaned.push(' ');
        StripState::Code
    } else {
        cleaned.push(' ');
        StripState::String { escaped: false }
    }
}

fn handle_char_strip_state(cleaned: &mut String, ch: char, escaped: bool) -> StripState {
    if ch == '\n' {
        cleaned.push('\n');
        StripState::Code
    } else if escaped {
        cleaned.push(' ');
        StripState::Char { escaped: false }
    } else if ch == '\\' {
        cleaned.push(' ');
        StripState::Char { escaped: true }
    } else if ch == '\'' {
        cleaned.push(' ');
        StripState::Code
    } else {
        cleaned.push(' ');
        StripState::Char { escaped: false }
    }
}

fn starts_rust_char_literal(mut remaining: std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    match (remaining.next(), remaining.next(), remaining.next()) {
        (Some('\\'), Some(_escaped), Some('\'')) => true,
        (Some(_value), Some('\''), _other) => true,
        _other => false,
    }
}

#[derive(Clone, Copy)]
enum StripState {
    Code,
    LineComment,
    BlockComment,
    String { escaped: bool },
    Char { escaped: bool },
}

fn is_source_scan_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("rs"))
        || path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml")
}

fn classify_manifest_section(line: &str) -> ManifestSection {
    let normalized = line.trim_start_matches('[').trim_end_matches(']');
    direct_manifest_section(normalized)
        .or_else(|| dependency_manifest_section(normalized))
        .unwrap_or(ManifestSection::None)
}

fn direct_manifest_section(normalized: &str) -> Option<ManifestSection> {
    if normalized == "package" {
        Some(ManifestSection::Package)
    } else if normalized == "lib" {
        Some(ManifestSection::Lib)
    } else {
        None
    }
}

fn dependency_manifest_section(normalized: &str) -> Option<ManifestSection> {
    [
        ("build-dependencies", ManifestSection::BuildDependencies),
        ("dev-dependencies", ManifestSection::DevDependencies),
        ("dependencies", ManifestSection::Dependencies),
    ]
    .into_iter()
    .find_map(|(section_name, section)| {
        manifest_section_matches(normalized, section_name).then_some(section)
    })
}

fn manifest_section_matches(normalized: &str, section_name: &str) -> bool {
    normalized == section_name
        || normalized.starts_with(&format!("{section_name}."))
        || normalized.ends_with(&format!(".{section_name}"))
        || normalized.contains(&format!(".{section_name}."))
}

fn dependency_name_from_section(line: &str, section_name: &str) -> Option<String> {
    let normalized = line.trim_start_matches('[').trim_end_matches(']');

    if let Some(name) = normalized.strip_prefix(&format!("{section_name}.")) {
        return Some(name.to_string());
    }

    let needle = format!(".{section_name}.");
    if let Some((_, name)) = normalized.rsplit_once(&needle) {
        return Some(name.to_string());
    }

    None
}

fn parse_manifest_value(line: &str, key: &str) -> Option<String> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }

    let value = rhs.trim().trim_matches('"');
    Some(value.to_string())
}

fn parse_manifest_bool(line: &str, key: &str) -> Option<bool> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }

    match rhs.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _other => None,
    }
}

fn parse_dependency_key(raw_line: &str, trimmed_line: &str) -> Option<String> {
    if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
        return None;
    }
    if trimmed_line.starts_with('#') || trimmed_line.starts_with('[') {
        return None;
    }
    let (lhs, _) = trimmed_line.split_once('=')?;
    Some(lhs.trim().to_string())
}

fn scan_join_handle_discard(documents: &[SourceDocument], max_evidence: usize) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        for (idx, (source_line, scan_line)) in document
            .contents
            .lines()
            .zip(document.risk_contents.lines())
            .enumerate()
        {
            let lowered = scan_line.to_ascii_lowercase();
            if JOIN_HANDLE_DISCARD_SPAWN_PATTERNS
                .iter()
                .any(|pattern| lowered.contains(pattern))
                && JOIN_HANDLE_DISCARD_CONTEXT_PATTERNS
                    .iter()
                    .any(|pattern| lowered.contains(pattern))
            {
                total_hits += 1;
                matched_patterns.insert("discarded JoinHandle");
                if evidence.len() < max_evidence {
                    evidence.push(ScanEvidence {
                        path: document.relative_path.clone(),
                        line_number: idx + 1,
                        pattern: "discarded JoinHandle",
                        snippet: source_line.trim().to_string(),
                    });
                }
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn scan_dependency_version_drift(documents: &[SourceDocument], max_evidence: usize) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents.iter().filter(|document| {
        document
            .relative_path
            .file_name()
            .and_then(|name| name.to_str())
            == Some("Cargo.toml")
    }) {
        for (idx, line) in document.contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let lowered = trimmed.to_ascii_lowercase();
            let Some((_, rhs)) = trimmed.split_once('=') else {
                continue;
            };
            let version_text = rhs.trim().trim_matches('"').to_ascii_lowercase();
            let is_manifest_inline_version = lowered.contains("version =");
            let is_plain_dependency = !trimmed.starts_with('[')
                && !trimmed.starts_with(' ')
                && !trimmed.starts_with('\t');

            if !(is_manifest_inline_version || is_plain_dependency) {
                continue;
            }

            let pattern = if version_text == "*" {
                "wildcard version"
            } else if version_text.contains(">=") && !version_text.contains('<') {
                "open-ended >= version"
            } else {
                continue;
            };

            total_hits += 1;
            matched_patterns.insert(pattern);
            if evidence.len() < max_evidence {
                evidence.push(ScanEvidence {
                    path: document.relative_path.clone(),
                    line_number: idx + 1,
                    pattern,
                    snippet: trimmed.to_string(),
                });
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn scan_global_shared_resource_patterns(
    documents: &[SourceDocument],
    max_evidence: usize,
) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        for (idx, (source_line, scan_line)) in document
            .contents
            .lines()
            .zip(document.risk_contents.lines())
            .enumerate()
        {
            let trimmed = scan_line.trim_start();
            let Some(pattern) = global_shared_pattern_for_line(trimmed) else {
                continue;
            };

            total_hits += 1;
            matched_patterns.insert(pattern);
            if evidence.len() < max_evidence {
                evidence.push(ScanEvidence {
                    path: document.relative_path.clone(),
                    line_number: idx + 1,
                    pattern,
                    snippet: source_line.trim().to_string(),
                });
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn global_shared_pattern_for_line(trimmed: &str) -> Option<&'static str> {
    let lowered = trimmed.to_ascii_lowercase();

    if trimmed.starts_with("static ")
        || trimmed.starts_with("pub static ")
        || trimmed.starts_with("pub(crate) static ")
    {
        Some("static declaration")
    } else if lowered.contains("static mut") {
        Some("static mut")
    } else if lowered.contains("lazy_static!") {
        Some("lazy_static!")
    } else if lowered.contains("oncecell::sync::lazy") {
        Some("oncecell::sync::lazy")
    } else if lowered.contains("oncelock<") {
        Some("oncelock<")
    } else if lowered.contains("lazylock<") {
        Some("lazylock<")
    } else {
        None
    }
}

fn scan_restricted_pointer_use(documents: &[SourceDocument], max_evidence: usize) -> PatternScan {
    let mut total_hits = 0usize;
    let mut evidence = Vec::new();
    let mut matched_patterns = BTreeSet::new();

    for document in documents {
        for (idx, (source_line, scan_line)) in document
            .contents
            .lines()
            .zip(document.risk_contents.lines())
            .enumerate()
        {
            let trimmed = scan_line.trim();
            if !is_code_like_scan_line(trimmed) {
                continue;
            }

            let lowered = trimmed.to_ascii_lowercase();
            let pattern = if lowered.contains("*const ") {
                "*const"
            } else if lowered.contains("*mut ") {
                "*mut"
            } else if lowered.contains("nonnull<") {
                "nonnull<"
            } else if lowered.contains("addr_of!(") {
                "addr_of!"
            } else if lowered.contains("extern \"c\" fn") {
                "extern fn"
            } else {
                continue;
            };

            total_hits += 1;
            matched_patterns.insert(pattern);
            if evidence.len() < max_evidence {
                evidence.push(ScanEvidence {
                    path: document.relative_path.clone(),
                    line_number: idx + 1,
                    pattern,
                    snippet: source_line.trim().to_string(),
                });
            }
        }
    }

    PatternScan {
        total_hits,
        matched_patterns: matched_patterns.into_iter().collect(),
        evidence,
    }
}

fn has_file_with_prefix(files: &[PathBuf], prefix: &str) -> bool {
    files.iter().any(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase().starts_with(prefix))
            .unwrap_or(false)
    })
}

fn has_exact_file_name(files: &[PathBuf], name: &str) -> bool {
    files.iter().any(|path| {
        path.file_name()
            .and_then(|file_name| file_name.to_str())
            .map(|file_name| file_name.eq_ignore_ascii_case(name))
            .unwrap_or(false)
    })
}

fn has_path_component(path: &Path, component: &str) -> bool {
    path.components().any(|part| {
        part.as_os_str()
            .to_str()
            .map(|value| value.eq_ignore_ascii_case(component))
            .unwrap_or(false)
    })
}

fn compute_tree_sha256(root: &Path, files: &[PathBuf]) -> io::Result<String> {
    let mut hasher = Sha256::new();

    for relative_path in files {
        let path_text = relative_path.display().to_string();
        hasher.update(path_text.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(path_text.as_bytes());
        hasher.update(b":");

        let contents = fs::read(root.join(relative_path))?;
        hasher.update(contents.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(&contents);
        hasher.update(b"\n");
    }

    Ok(hex_encode(&hasher.finalize()))
}

fn scan_vcs_info(root: &Path) -> VcsInfo {
    let path = root.join(".cargo_vcs_info.json");
    let Ok(contents) = fs::read_to_string(path) else {
        return VcsInfo::default();
    };
    let Ok(value) = serde_json::from_str::<Value>(&contents) else {
        return VcsInfo::default();
    };

    VcsInfo {
        git_commit: value
            .get("git")
            .and_then(|git| git.get("sha1"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        path_in_vcs: value
            .get("path_in_vcs")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn scan_artifact_stem(report: &CrateSourceScanReport) -> String {
    format!("{}_scan", sanitize_filename_component(&report.crate_name))
}

fn prepare_scan_output_run_at(
    base_output_root: &Path,
    timestamp_utc: &str,
) -> io::Result<ScanRunPaths> {
    fs::create_dir_all(base_output_root)?;
    let run_dir = create_unique_run_dir(base_output_root, &format!("dsfb-gray-{timestamp_utc}"))?;
    Ok(ScanRunPaths {
        base_output_root: base_output_root.to_path_buf(),
        run_dir,
        timestamp_utc: timestamp_utc.to_string(),
    })
}

fn create_unique_run_dir(base_output_root: &Path, base_name: &str) -> io::Result<PathBuf> {
    let primary = base_output_root.join(base_name);
    if !primary.exists() {
        fs::create_dir_all(&primary)?;
        return Ok(primary);
    }

    for suffix in 1..=999 {
        let candidate = base_output_root.join(format!("{base_name}-{suffix:02}"));
        if !candidate.exists() {
            fs::create_dir_all(&candidate)?;
            return Ok(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!("unable to create unique scan output directory for {base_name}"),
    ))
}

fn scan_run_timestamp(now: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn collect_legacy_scan_artifacts(legacy_root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(legacy_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if is_legacy_scan_artifact(file_name) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn is_legacy_scan_artifact(file_name: &str) -> bool {
    file_name.ends_with("_scan.txt")
        || file_name.ends_with("_scan.sarif.json")
        || file_name.ends_with("_scan.intoto.json")
        || file_name.ends_with("_scan.dsse.json")
}

fn sanitize_filename_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
        } else {
            sanitized.push('_');
        }
    }
    sanitized.trim_matches('_').to_string()
}

fn parse_secret_key(secret: &str) -> io::Result<[u8; 32]> {
    let trimmed = secret.trim();
    let bytes = if trimmed.len() == 64 && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        hex_decode(trimmed)?
    } else {
        BASE64_STANDARD.decode(trimmed).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid base64 signing key: {err}"),
            )
        })?
    };

    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "signing key must decode to exactly 32 bytes",
        )
    })
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter().copied() {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

fn hex_decode(text: &str) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len() / 2);
    let mut pairs = text.as_bytes().chunks_exact(2);
    for pair in &mut pairs {
        let high = decode_hex_nibble(pair[0])?;
        let low = decode_hex_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn decode_hex_nibble(ch: u8) -> io::Result<u8> {
    match ch {
        b'0'..=b'9' => Ok(ch - b'0'),
        b'a'..=b'f' => Ok(ch - b'a' + 10),
        b'A'..=b'F' => Ok(ch - b'A' + 10),
        _other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid hex digit in signing key",
        )),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn dsse_pae(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"DSSEv1 ");
    out.extend_from_slice(payload_type.len().to_string().as_bytes());
    out.push(b' ');
    out.extend_from_slice(payload_type.as_bytes());
    out.push(b' ');
    out.extend_from_slice(payload.len().to_string().as_bytes());
    out.push(b' ');
    out.extend_from_slice(payload);
    out
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scan_finds_async_and_channel_motifs() {
        let fixture_root = unique_fixture_root();
        let src_dir = fixture_root.join("src");
        fs::create_dir_all(&src_dir).expect("create fixture dir");
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture-scan\"\nversion = \"0.1.0\"\n",
        )
        .expect("write manifest");
        fs::write(
            src_dir.join("lib.rs"),
            "use tokio::sync::mpsc;\nuse tokio::task::spawn_blocking;\nfn f(){ let _ = mpsc::channel::<u8>(16); let _ = spawn_blocking(|| 1); }\n",
        )
        .expect("write source");

        let report = scan_crate_source(&fixture_root).expect("scan succeeds");
        let matched_ids: BTreeSet<&str> = report
            .matched_heuristics
            .iter()
            .map(|matched| matched.heuristic.id.0)
            .collect();

        assert!(matched_ids.contains("H-ASYNC-01"));
        assert!(matched_ids.contains("H-CHAN-01"));

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn scan_reports_certification_signals() {
        let fixture_root = unique_fixture_root();
        let src_dir = fixture_root.join("src");
        let tests_dir = fixture_root.join("tests");
        let docs_dir = fixture_root.join("docs");
        fs::create_dir_all(&src_dir).expect("create src dir");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        fs::create_dir_all(&docs_dir).expect("create docs dir");

        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture-cert\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\nrust-version = \"1.75\"\nrepository = \"https://example.invalid/repo\"\ndocumentation = \"https://docs.example.invalid\"\nhomepage = \"https://example.invalid\"\nreadme = \"README.md\"\n\n[dependencies]\nembedded-hal = \"1\"\n\n[dev-dependencies]\nproptest = \"1\"\n",
        )
        .expect("write manifest");
        fs::write(
            src_dir.join("lib.rs"),
            "#![no_std]\n#![forbid(unsafe_code)]\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn smoke() { assert_eq!(2 + 2, 4); }\n}\n",
        )
        .expect("write source");
        fs::write(
            tests_dir.join("prop.rs"),
            "use proptest::prelude::*;\nproptest! {\n    #[test]\n    fn roundtrip(x in 0u8..) { prop_assert_eq!(x, x); }\n}\n",
        )
        .expect("write property test");
        fs::write(
            src_dir.join("risky.rs"),
            "use core::cell::RefCell;\nfn control_path(state: Option<u8>) -> u8 {\n    let cell = RefCell::new(0u8);\n    match state { Some(v) => v, _ => *cell.borrow() }\n}\nfn wait_for_hw() { let _ = std::time::Duration::from_millis(5); }\n",
        )
        .expect("write risky source");
        fs::write(fixture_root.join("README.md"), "# Fixture\n").expect("write readme");
        fs::write(fixture_root.join("SAFETY.md"), "No unsafe code.\n").expect("write safety");
        fs::write(fixture_root.join("SECURITY.md"), "Security policy.\n").expect("write security");
        fs::write(fixture_root.join("LICENSE"), "MIT\n").expect("write license");
        fs::write(docs_dir.join("design.md"), "Design notes.\n").expect("write doc");

        let report = scan_crate_source(&fixture_root).expect("scan succeeds");

        assert!(report.certification.runtime.no_std_declared);
        assert_eq!(report.certification.runtime.alloc_crate_hits, 0);
        assert_eq!(report.certification.runtime.heap_allocation_hits, 0);
        assert_eq!(
            report.certification.safety.unsafe_policy,
            UnsafeCodePolicy::Forbid
        );
        assert_eq!(report.certification.safety.unsafe_sites, 0);
        assert!(report.certification.verification.tests_dir_present);
        assert!(report.certification.verification.property_testing_hits > 0);
        assert!(report.certification.lifecycle.safety_md_present);
        assert_eq!(report.certification.build.direct_dependencies, 1);
        assert_eq!(report.certification.build.dev_dependencies, 1);

        let rendered = render_scan_report(&report);
        assert!(rendered.contains("Audit Summary"));
        assert!(rendered.contains("Add dsfb-gray report badge to your GitHub repo README"));
        assert!(rendered.contains("Overall Score and Subscores"));
        assert!(rendered.contains("Scanner Crate: https://crates.io/crates/dsfb-gray"));
        assert!(rendered.contains("DSFB-gray crate: https://crates.io/crates/dsfb-gray"));
        assert!(rendered.contains("https://img.shields.io/badge/DSFB%20Gray%20Audit-"));
        assert!(rendered.contains("[![DSFB Gray Audit:"));
        assert!(rendered.contains("(./fixture_cert_scan.txt)"));
        assert!(rendered.contains("Score Summary Table"));
        assert!(rendered.contains("Advisory Broad Subscores"));
        assert!(rendered
            .contains("| Section                      | Score% | Weight | Points | Checks |"));
        assert!(rendered.contains("Scoring guideline:"));
        assert!(rendered.contains("not a compliance certification"));
        assert!(rendered.contains("no_std declared: yes"));
        assert!(rendered.contains("no_alloc candidate: yes"));
        assert!(rendered.contains("no_unsafe candidate: yes"));
        assert!(rendered.contains("NASA/JPL Power of Ten Audit"));
        assert!(rendered.contains("Advanced Structural Risk Checks"));
        assert!(rendered.contains("Top Findings"));
        assert!(rendered.contains("Code Quality Themes"));
        assert!(rendered.contains("Verification Suggestions"));
        assert!(rendered.contains("Evidence Ledger"));
        assert!(rendered.contains("Conclusion Lenses"));
        assert!(rendered.contains("Criticality Heatmap"));
        assert!(rendered.contains("row format: path:line `function` [bar]"));
        assert!(rendered.contains("JPL-R4 elevated"));
        assert!(rendered.contains("TIME-WAIT elevated"));
        assert!(rendered.contains("Remediation:"));
        assert!(rendered.contains("TIME-WAIT-01"));

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn scan_renders_structured_attestations() {
        let fixture_root = unique_fixture_root();
        let src_dir = fixture_root.join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");

        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture-attest\"\nversion = \"0.1.0\"\n",
        )
        .expect("write manifest");
        fs::write(
            fixture_root.join(".cargo_vcs_info.json"),
            "{ \"git\": { \"sha1\": \"deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\" }, \"path_in_vcs\": \"fixture-attest\" }\n",
        )
        .expect("write vcs info");
        fs::write(
            src_dir.join("lib.rs"),
            "use tokio::sync::mpsc;\nfn f(){ let _ = mpsc::channel::<u8>(4); }\n",
        )
        .expect("write source");

        let report = scan_crate_source(&fixture_root).expect("scan succeeds");
        let rendered = render_scan_report(&report);
        assert!(rendered.contains("Generated At (UTC):"));
        assert!(rendered.contains("Source SHA-256:"));
        assert!(rendered.contains("Scoring Version: dsfb-assurance-score-v1"));
        assert!(rendered.contains("Markdown snippet:"));
        assert!(rendered.contains(
            "Treat this report as a structured guideline for improvement and review readiness."
        ));
        assert!(rendered.contains("Conclusion Lenses"));

        let sarif_json = render_scan_sarif(&report);
        let sarif_value: Value = serde_json::from_str(&sarif_json).expect("parse sarif");
        assert_eq!(sarif_value["version"], "2.1.0");
        assert_eq!(
            sarif_value["runs"][0]["properties"]["sourceSha256"],
            report.source_sha256
        );
        assert_eq!(
            sarif_value["runs"][0]["properties"]["auditScore"]["method"],
            AUDIT_SCORE_METHOD
        );
        assert_eq!(
            sarif_value["runs"][0]["properties"]["auditMode"],
            "canonical-broad-audit"
        );
        assert!(sarif_value["runs"][0]["properties"]["guidanceSemantics"]
            ["nonCertificationStatement"]
            .as_str()
            .expect("non-cert statement")
            .contains("does not certify compliance"));
        assert_eq!(
            sarif_value["runs"][0]["tool"]["driver"]["rules"][0]["help"]["text"]
                .as_str()
                .expect("rule help"),
            heuristic_remediation("H-CHAN-01")
        );

        let statement_json = render_scan_attestation_statement(&report);
        let statement_value: Value =
            serde_json::from_str(&statement_json).expect("parse statement");
        assert_eq!(statement_value["_type"], "https://in-toto.io/Statement/v1");
        assert_eq!(
            statement_value["subject"][0]["digest"]["sha256"],
            report.source_sha256
        );
        assert_eq!(
            statement_value["predicate"]["summary"]["auditScore"]["method"],
            AUDIT_SCORE_METHOD
        );
        assert_eq!(
            statement_value["predicate"]["scanner"]["auditMode"],
            "canonical-broad-audit"
        );
        assert!(
            statement_value["predicate"]["guidanceSemantics"]["nonCertificationStatement"]
                .as_str()
                .expect("predicate non-cert statement")
                .contains("does not certify compliance")
        );
        assert!(
            statement_value["predicate"]["summary"]["matchedHeuristics"][0]["structuralPrior"]
                .is_object()
        );

        let unsigned_dsse = render_scan_dsse_envelope(&report, None);
        let unsigned_value: Value =
            serde_json::from_str(&unsigned_dsse).expect("parse unsigned dsse");
        assert_eq!(unsigned_value["payloadType"], DSSE_PAYLOAD_TYPE);
        assert_eq!(
            unsigned_value["signatures"]
                .as_array()
                .expect("signatures array")
                .len(),
            0
        );

        let signer = ScanSigningKey::from_secret_text(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            Some("test-ed25519"),
        )
        .expect("create signer");
        let signed_dsse = render_scan_dsse_envelope(&report, Some(&signer));
        let signed_value: Value = serde_json::from_str(&signed_dsse).expect("parse signed dsse");
        assert_eq!(
            signed_value["signatures"]
                .as_array()
                .expect("signatures array")
                .len(),
            1
        );
        assert_eq!(signed_value["signatures"][0]["keyid"], "test-ed25519");

        let export_dir = fixture_root.join("exports");
        let paths = export_scan_artifacts(&report, &export_dir, Some(&signer))
            .expect("export artifacts succeeds");
        assert!(paths.signed);
        assert_eq!(paths.output_dir, export_dir);
        assert!(paths.report_path.exists());
        assert!(paths.sarif_path.exists());
        assert!(paths.statement_path.exists());
        assert!(paths.dsse_path.exists());
        assert_eq!(
            paths
                .report_path
                .file_name()
                .and_then(|value| value.to_str()),
            Some("fixture_attest_scan.txt")
        );

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn derived_static_priors_are_bounded() {
        let fixture_root = unique_fixture_root();
        let src_dir = fixture_root.join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture-prior\"\nversion = \"0.1.0\"\n",
        )
        .expect("write manifest");
        fs::write(
            src_dir.join("lib.rs"),
            "use tokio::sync::mpsc;\nuse tokio::task::spawn_blocking;\nfn f(){ let _ = mpsc::channel::<u8>(4); let _ = mpsc::channel::<u8>(8); let _ = spawn_blocking(|| 1); }\n",
        )
        .expect("write source");

        let report = scan_crate_source(&fixture_root).expect("scan succeeds");
        let priors = derive_static_priors_from_scan(&report);

        assert!(!priors.is_empty());
        let chan_prior = priors
            .get(crate::HeuristicId("H-CHAN-01"))
            .expect("channel prior");
        assert!(chan_prior.confidence >= 0.15);
        assert!(chan_prior.confidence <= 0.95);
        assert!(chan_prior.drift_scale >= 0.75);
        assert!(chan_prior.drift_scale <= 1.0);

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn scan_output_run_paths_are_timestamped_and_unique() {
        let fixture_root = unique_fixture_root();
        fs::create_dir_all(&fixture_root).expect("create fixture root");

        let first = prepare_scan_output_run_at(&fixture_root, "2026-04-14T01-23-45Z")
            .expect("create first run dir");
        let second = prepare_scan_output_run_at(&fixture_root, "2026-04-14T01-23-45Z")
            .expect("create second run dir");

        assert_eq!(first.base_output_root, fixture_root);
        assert_eq!(first.timestamp_utc, "2026-04-14T01-23-45Z");
        assert!(
            first.run_dir.ends_with("dsfb-gray-2026-04-14T01-23-45Z"),
            "unexpected first run dir: {}",
            first.run_dir.display()
        );
        assert!(
            second
                .run_dir
                .ends_with("dsfb-gray-2026-04-14T01-23-45Z-01"),
            "unexpected second run dir: {}",
            second.run_dir.display()
        );

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn legacy_scan_artifacts_migrate_once() {
        let fixture_root = unique_fixture_root();
        fs::create_dir_all(&fixture_root).expect("create fixture root");
        let output_root = fixture_root.join(DEFAULT_SCAN_OUTPUT_ROOT);

        for name in [
            "tokio_scan.txt",
            "tokio_scan.sarif.json",
            "tokio_scan.intoto.json",
            "tokio_scan.dsse.json",
        ] {
            fs::write(fixture_root.join(name), "fixture").expect("write legacy scan artifact");
        }

        let migration_dir = migrate_legacy_scan_artifacts(&fixture_root, &output_root)
            .expect("migrate legacy scan artifacts")
            .expect("migration dir");
        assert!(migration_dir.exists());
        for name in [
            "tokio_scan.txt",
            "tokio_scan.sarif.json",
            "tokio_scan.intoto.json",
            "tokio_scan.dsse.json",
        ] {
            assert!(
                migration_dir.join(name).exists(),
                "missing migrated file {name}"
            );
            assert!(
                !fixture_root.join(name).exists(),
                "legacy file still present {name}"
            );
        }

        assert!(migrate_legacy_scan_artifacts(&fixture_root, &output_root)
            .expect("rerun migration")
            .is_none());

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn scan_reports_extended_structural_audits() {
        let fixture_root = unique_fixture_root();
        let src_dir = fixture_root.join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");

        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture-extended\"\nversion = \"0.1.0\"\n\n[dependencies]\ntokio = \"*\"\nbytes = { version = \">=1\" }\n",
        )
        .expect("write manifest");
        fs::write(
            src_dir.join("lib.rs"),
            "use core::future::Future;\nuse core::pin::Pin;\nuse core::sync::atomic::{AtomicUsize, Ordering};\nuse core::task::{Context, Poll};\nuse std::io::Write;\n\n#[interrupt]\nfn irq() {\n    let _ = Box::new(1u8);\n}\n\nfn iter_unbounded(items: impl Iterator<Item = u8>) -> usize {\n    items.collect::<Vec<_>>().len()\n}\n\nstruct PendingFuture;\nimpl Future for PendingFuture {\n    type Output = ();\n    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {\n        Poll::Pending\n    }\n}\n\nasync fn launch() {\n    let _ = tokio::spawn(async {});\n    let _ = tokio::task::spawn_blocking(|| 1usize);\n}\n\nstruct Boom;\nimpl Drop for Boom {\n    fn drop(&mut self) {\n        panic!(\"boom\");\n    }\n}\n\nfn leader_state(counter: &AtomicUsize) -> usize {\n    let state = counter.load(Ordering::Relaxed);\n    if state > 0 { state } else { 0 }\n}\n\nfn mixed_clock() {\n    let _ = std::time::Instant::now();\n    let _ = std::time::SystemTime::now();\n}\n\nfn short_write(w: &mut dyn Write, buf: &[u8]) {\n    let _ = w.write(buf);\n}\n\n#[async_recursion]\nasync fn recurse() {\n    recurse().await;\n}\n\nfn queue() {\n    let _ = tokio::sync::mpsc::unbounded_channel::<u8>();\n}\n\nasync fn read_packet(packet: &[u8]) {\n    let payload = packet.to_vec();\n    drop(payload);\n}\n",
        )
        .expect("write source");

        let report = scan_crate_source(&fixture_root).expect("scan succeeds");
        let rendered = render_scan_report(&report);

        for expected in [
            "ITER-UNB elevated",
            "ISR-SAFE elevated",
            "FUTURE-WAKE elevated",
            "TASK-LEAK elevated",
            "DROP-PANIC elevated",
            "ATOMIC-RELAXED elevated",
            "CLOCK-MIX elevated",
            "SHORT-WRITE elevated",
            "ASYNC-RECUR elevated",
            "CHAN-UNB elevated",
            "ZERO-COPY elevated",
            "CARGO-VERS elevated",
        ] {
            assert!(
                rendered.contains(expected),
                "missing expected check {expected}"
            );
        }

        let _ = fs::remove_dir_all(&fixture_root);
    }

    fn unique_fixture_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "dsfb-gray-scan-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before epoch")
                .as_nanos()
        ))
    }
}
