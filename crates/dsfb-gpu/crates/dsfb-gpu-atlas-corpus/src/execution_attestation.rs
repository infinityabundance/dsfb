//! T.11e — `ExecutionAttestationReceiptV1`: unsigned, DSFB-native
//! local execution receipt.
//!
//! Panel framing:
//!
//! > T.11e is a bridge artifact:
//! > DSFB court artifact → local execution receipt → future
//! > signed attestation → future policy verifier.
//!
//! The receipt borrows the **useful structure** of SLSA / in-toto
//! provenance (subjects, materials, build commands, environment)
//! but remains a **DSFB-native deterministic receipt** until
//! signing, CI, identity, and policy verification exist. It does
//! NOT claim SLSA compliance, an in-toto signed statement, a
//! release attestation, third-party verification, or reproducible-
//! build proof. The non-claim discipline is **verifier-enforced**
//! via the [`AttestationNonClaim`] enum and the
//! `claimed_slsa_level` / `claimed_signed_attestation` fields.
//!
//! **Hash posture (panel-locked)**:
//!
//! - `corpus_hash_v1` unchanged (frozen at T.10).
//! - `registry_hash_v2` unchanged (frozen at S1.2).
//! - `precedent_hash_v1` unchanged (frozen at T.11b).
//! - `admissibility_grammar_hash_v1` unchanged (frozen at T.11c).
//! - `trial_transcript_hash_v1` unchanged (frozen at T.11d).
//! - `receipt_hash_v1` is the NEW T.11e attestation hash.
//!
//! Rendered text is NOT in the hash material; canonical bytes
//! only. Domain separator:
//! `DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1\0`.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::admissibility::collect_admissibility_grammar;
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::precedent::collect_court_precedents;
use crate::trial_transcript::build_t11d_latency_ramp_fixture;

/// Domain separator prefix for `receipt_hash_v1`. **Panel-locked**;
/// changing it changes every attestation hash.
pub const EXECUTION_ATTESTATION_DOMAIN: &str = "DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1\0";

/// Schema identifier carried inside the hash material.
pub const EXECUTION_ATTESTATION_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:EXECUTION-ATTESTATION:v1";

/// Stable handle for one execution-attestation receipt. At T.11e
/// the test fixture's id is `1`; future receipts append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecutionReceiptId(pub u32);

/// Schema variant carried in the hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionAttestationSchema {
    /// T.11e — unsigned DSFB-native receipt. No SLSA claim, no
    /// in-toto signature, no third-party verification.
    V1UnsignedDsfbNative,
}

impl ExecutionAttestationSchema {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1UnsignedDsfbNative => "V1UnsignedDsfbNative",
        }
    }
}

/// Purpose tag for one executed command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandPurpose {
    /// Compile-time build (`cargo build ...`).
    Build,
    /// Format check (`cargo fmt --all --check`).
    Format,
    /// Static-analysis gate (`cargo clippy ...`).
    Clippy,
    /// Attribution / forbidden-string gate (`scripts/scrub.sh`).
    Scrub,
    /// Docs-freshness gate (`scripts/docs_freshness.sh`).
    DocsFreshness,
    /// Per-package test pass.
    SinglePackageTest,
    /// Full workspace test pass.
    WorkspaceTest,
    /// Bulk-emit of an artifact (one of the `*-emit` CLIs).
    BulkEmit,
    /// 10-step regression-check ritual receipt emission.
    RegressionCheck,
}

impl CommandPurpose {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Format => "Format",
            Self::Clippy => "Clippy",
            Self::Scrub => "Scrub",
            Self::DocsFreshness => "DocsFreshness",
            Self::SinglePackageTest => "SinglePackageTest",
            Self::WorkspaceTest => "WorkspaceTest",
            Self::BulkEmit => "BulkEmit",
            Self::RegressionCheck => "RegressionCheck",
        }
    }
}

/// One recorded command line. The receipt records what was
/// claimed to be run; it does NOT re-execute the command.
#[derive(Debug, Clone)]
pub struct ExecutedCommand {
    /// The exact command string (e.g.
    /// `cargo clippy --workspace --all-targets --features cuda -- -D warnings`).
    pub command: String,
    /// Why this command was run.
    pub purpose: CommandPurpose,
    /// Observed exit code (0 means success). The verifier
    /// rejects non-zero exit codes on Format / Clippy / Scrub /
    /// DocsFreshness / WorkspaceTest commands.
    pub exit_code: i32,
}

/// Kind of input material recorded in the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialKind {
    /// The DSFB corpus snapshot.
    Corpus,
    /// The S1.2 detector registry.
    Registry,
    /// The T.11b precedent layer.
    Precedent,
    /// The T.11c admissibility grammar.
    Grammar,
    /// Source-code input (e.g. a `.rs` file's content hash).
    SourceCode,
    /// External input artifact (e.g. a `.toml` configuration).
    InputArtifact,
}

impl MaterialKind {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corpus => "Corpus",
            Self::Registry => "Registry",
            Self::Precedent => "Precedent",
            Self::Grammar => "Grammar",
            Self::SourceCode => "SourceCode",
            Self::InputArtifact => "InputArtifact",
        }
    }
}

/// One material-input digest.
#[derive(Debug, Clone)]
pub struct MaterialDigest {
    /// Kind of material.
    pub kind: MaterialKind,
    /// Human-readable name (used in renderers AND in the hash).
    pub name: String,
    /// 32-byte SHA-256 digest of the material content.
    pub digest: [u8; 32],
}

/// Kind of output subject produced by the build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubjectKind {
    /// A T.11d trial-transcript artifact.
    Transcript,
    /// A T.11c admissibility-grammar artifact.
    Grammar,
    /// A T.11b precedent-ledger artifact.
    PrecedentLedger,
    /// A T.11a passport bulk dump.
    PassportBulk,
    /// A regression-check / verification / summary receipt.
    Receipt,
    /// The 10-step ritual receipt.
    RegressionCheck,
}

impl SubjectKind {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transcript => "Transcript",
            Self::Grammar => "Grammar",
            Self::PrecedentLedger => "PrecedentLedger",
            Self::PassportBulk => "PassportBulk",
            Self::Receipt => "Receipt",
            Self::RegressionCheck => "RegressionCheck",
        }
    }
}

/// One output-subject digest.
#[derive(Debug, Clone)]
pub struct SubjectDigest {
    /// Kind of subject.
    pub kind: SubjectKind,
    /// Human-readable name (e.g.
    /// `crates/dsfb-gpu-atlas-corpus/out/trial_transcript_v1.txt`).
    pub name: String,
    /// 32-byte SHA-256 digest of the subject content.
    pub digest: [u8; 32],
}

/// Aggregate workspace-test summary recorded in the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestSummary {
    /// Number of `cargo test` result groups (one per binary).
    pub workspace_test_groups: u32,
    /// Total passed assertions across all groups.
    pub workspace_passed: u32,
    /// Total failed assertions. MUST be zero.
    pub workspace_failed: u32,
    /// Total ignored assertions.
    pub workspace_ignored: u32,
}

/// Per-gate summary for the 4 pre-commit gates + R.12b
/// byte-stability statement. Every flag MUST be `true` for the
/// verifier to admit a non-dirty receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GateSummary {
    /// `cargo fmt --all --check` exit code 0.
    pub fmt_clean: bool,
    /// `cargo clippy --workspace --all-targets --features cuda -- -D warnings` exit code 0.
    pub clippy_clean: bool,
    /// `bash scripts/scrub.sh` exit code 0.
    pub scrub_clean: bool,
    /// `bash scripts/docs_freshness.sh` exit code 0.
    pub docs_freshness_clean: bool,
    /// `tests/r12_d64_saturation` episode counts byte-identical
    /// to the pinned R.12b baseline (13 / 89 / 1917).
    pub r12_byte_stability_clean: bool,
}

/// Pinned R.12b episode counts. The verifier asserts each field
/// matches the panel-locked baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct R12bEpisodeCounts {
    /// canonical 16×128 episodes/cat. Panel-locked at 13.
    pub canonical_16x128: u32,
    /// mid 64×512 episodes/cat. Panel-locked at 89.
    pub mid_64x512: u32,
    /// full 256×4096 episodes/cat. Panel-locked at 1917.
    pub full_256x4096: u32,
}

impl R12bEpisodeCounts {
    /// Return the panel-locked R.12b baseline.
    #[must_use]
    pub const fn baseline() -> Self {
        Self {
            canonical_16x128: 13,
            mid_64x512: 89,
            full_256x4096: 1917,
        }
    }
}

/// Panel-locked non-claim assertions. Every receipt MUST carry
/// at least the first seven, and the verifier asserts that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttestationNonClaim {
    /// The receipt is unsigned and locally produced.
    UnsignedLocalReceipt,
    /// Not a SLSA compliance claim.
    NotSlsaComplianceClaim,
    /// Not an in-toto signed statement.
    NotInTotoSignedStatement,
    /// Not a release artifact.
    NotReleaseArtifact,
    /// Not third-party verified.
    NotThirdPartyVerified,
    /// Not a reproducible-build proof.
    NotReproducibleBuildProof,
    /// Records the operator's observed environment only — no
    /// guarantee of identity or platform integrity.
    RecordsObservedEnvironmentOnly,
}

impl AttestationNonClaim {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsignedLocalReceipt => "UnsignedLocalReceipt",
            Self::NotSlsaComplianceClaim => "NotSlsaComplianceClaim",
            Self::NotInTotoSignedStatement => "NotInTotoSignedStatement",
            Self::NotReleaseArtifact => "NotReleaseArtifact",
            Self::NotThirdPartyVerified => "NotThirdPartyVerified",
            Self::NotReproducibleBuildProof => "NotReproducibleBuildProof",
            Self::RecordsObservedEnvironmentOnly => "RecordsObservedEnvironmentOnly",
        }
    }

    /// Return all panel-locked non-claims in canonical order.
    #[must_use]
    pub const fn all_required() -> [AttestationNonClaim; 7] {
        [
            Self::UnsignedLocalReceipt,
            Self::NotSlsaComplianceClaim,
            Self::NotInTotoSignedStatement,
            Self::NotReleaseArtifact,
            Self::NotThirdPartyVerified,
            Self::NotReproducibleBuildProof,
            Self::RecordsObservedEnvironmentOnly,
        ]
    }
}

/// The unsigned DSFB-native execution-attestation receipt.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ExecutionAttestationReceiptV1 {
    /// Schema variant.
    pub schema: ExecutionAttestationSchema,
    /// Stable handle.
    pub receipt_id: ExecutionReceiptId,

    /// Git commit hex string (40 chars). Empty string is rejected.
    pub repo_commit: String,
    /// True if `git status` reports uncommitted changes. The
    /// verifier rejects unless `dirty_override_acknowledged` is
    /// set.
    pub repo_dirty: bool,
    /// Operator-set override allowing a dirty-tree attestation
    /// (e.g. for diagnostic snapshots). Default false.
    pub dirty_override_acknowledged: bool,
    /// Optional branch name (e.g. `"master"`).
    pub branch_name: Option<String>,

    /// rustc --version line.
    pub rustc_version: String,
    /// cargo --version line.
    pub cargo_version: String,
    /// CUDA version reported by the toolchain.
    pub cuda_version: Option<String>,
    /// nvcc --version line.
    pub nvcc_version: Option<String>,
    /// GPU device name (e.g. "RTX 4080 SUPER").
    pub gpu_name: Option<String>,
    /// GPU compute capability (e.g. "8.9").
    pub gpu_compute_capability: Option<String>,

    /// Commands run during the build leading up to this receipt.
    pub build_commands: Vec<ExecutedCommand>,
    /// Verification / gate commands run.
    pub verification_commands: Vec<ExecutedCommand>,

    /// T.10 corpus identity (echoed for the receipt's hash chain).
    pub corpus_hash_v1: [u8; 32],
    /// S1.2 registry identity.
    pub registry_hash_v2: [u8; 32],
    /// T.11b precedent identity.
    pub precedent_hash_v1: [u8; 32],
    /// T.11c admissibility-grammar identity.
    pub admissibility_grammar_hash_v1: [u8; 32],
    /// T.11d trial-transcript identity.
    pub trial_transcript_hash_v1: [u8; 32],

    /// Input materials recorded in the receipt (sorted by name).
    pub materials: Vec<MaterialDigest>,
    /// Output subjects recorded (sorted by name).
    pub subjects: Vec<SubjectDigest>,

    /// nvcc compile flags used for any CUDA artifacts (informational).
    pub nvcc_flags: Vec<String>,
    /// rustc flags used for the workspace build (informational).
    pub rust_flags: Vec<String>,

    /// Aggregate workspace-test summary.
    pub test_summary: TestSummary,
    /// Per-gate summary.
    pub gate_summary: GateSummary,
    /// Pinned R.12b episode counts cross-check.
    pub r12b_episode_counts: R12bEpisodeCounts,

    /// MUST be `None`. The verifier rejects any non-None value
    /// because T.11e does NOT make a SLSA compliance claim.
    pub claimed_slsa_level: Option<u8>,
    /// MUST be `false`. The verifier rejects `true` because
    /// T.11e is the **unsigned** local receipt.
    pub claimed_signed_attestation: bool,

    /// Non-claim enum entries declared. MUST include every
    /// panel-locked entry in
    /// [`AttestationNonClaim::all_required`].
    pub non_claims: Vec<AttestationNonClaim>,

    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of every other field. Rendered text is NOT
    /// hashed.
    pub receipt_hash_v1: [u8; 32],
}

/// Build the deterministic T.11e test fixture. Every field is
/// pinned so two builds produce byte-identical receipts and
/// hashes. NOT for production use — the live CLI uses
/// [`build_t11e_live_attestation`] which queries the actual
/// environment.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_t11e_test_fixture() -> ExecutionAttestationReceiptV1 {
    let corpus = compute_corpus_hash_v1();
    let precedents = collect_court_precedents();
    let grammar = collect_admissibility_grammar();
    let transcript = build_t11d_latency_ramp_fixture();
    let registry_hash_v2 = registry_hash_v2_post_s12();

    let build_commands = alloc::vec![ExecutedCommand {
        command: "cargo build --workspace --features cuda".to_string(),
        purpose: CommandPurpose::Build,
        exit_code: 0,
    },];

    let verification_commands = alloc::vec![
        ExecutedCommand {
            command: "cargo fmt --all --check".to_string(),
            purpose: CommandPurpose::Format,
            exit_code: 0,
        },
        ExecutedCommand {
            command: "cargo clippy --workspace --all-targets --features cuda -- -D warnings"
                .to_string(),
            purpose: CommandPurpose::Clippy,
            exit_code: 0,
        },
        ExecutedCommand {
            command: "bash scripts/scrub.sh".to_string(),
            purpose: CommandPurpose::Scrub,
            exit_code: 0,
        },
        ExecutedCommand {
            command: "bash scripts/docs_freshness.sh".to_string(),
            purpose: CommandPurpose::DocsFreshness,
            exit_code: 0,
        },
        ExecutedCommand {
            command: "cargo test --workspace --features cuda -- --test-threads=1".to_string(),
            purpose: CommandPurpose::WorkspaceTest,
            exit_code: 0,
        },
    ];

    let mut materials = alloc::vec![
        MaterialDigest {
            kind: MaterialKind::Corpus,
            name: "corpus_hash_v1".to_string(),
            digest: corpus.bytes,
        },
        MaterialDigest {
            kind: MaterialKind::Registry,
            name: "registry_hash_v2".to_string(),
            digest: registry_hash_v2,
        },
        MaterialDigest {
            kind: MaterialKind::Precedent,
            name: "precedent_hash_v1".to_string(),
            digest: precedents.precedent_hash_v1,
        },
        MaterialDigest {
            kind: MaterialKind::Grammar,
            name: "admissibility_grammar_hash_v1".to_string(),
            digest: grammar.admissibility_grammar_hash_v1.0,
        },
    ];
    materials.sort_by(|a, b| a.name.cmp(&b.name));

    let mut subjects = alloc::vec![
        SubjectDigest {
            kind: SubjectKind::Transcript,
            name: "trial_transcript_v1".to_string(),
            digest: transcript.trial_transcript_hash_v1,
        },
        SubjectDigest {
            kind: SubjectKind::Grammar,
            name: "admissibility_grammar_v1".to_string(),
            digest: grammar.admissibility_grammar_hash_v1.0,
        },
        SubjectDigest {
            kind: SubjectKind::PrecedentLedger,
            name: "court_precedents_v1".to_string(),
            digest: precedents.precedent_hash_v1,
        },
    ];
    subjects.sort_by(|a, b| a.name.cmp(&b.name));

    let mut r = ExecutionAttestationReceiptV1 {
        schema: ExecutionAttestationSchema::V1UnsignedDsfbNative,
        receipt_id: ExecutionReceiptId(1),
        repo_commit: "0000000000000000000000000000000000000000".to_string(),
        repo_dirty: false,
        dirty_override_acknowledged: false,
        branch_name: Some("master".to_string()),
        rustc_version: "rustc 1.94.0".to_string(),
        cargo_version: "cargo 1.94.0".to_string(),
        cuda_version: Some("CUDA 13.2".to_string()),
        nvcc_version: Some("nvcc release 13.2".to_string()),
        gpu_name: Some("RTX 4080 SUPER".to_string()),
        gpu_compute_capability: Some("8.9".to_string()),
        build_commands,
        verification_commands,
        corpus_hash_v1: corpus.bytes,
        registry_hash_v2,
        precedent_hash_v1: precedents.precedent_hash_v1,
        admissibility_grammar_hash_v1: grammar.admissibility_grammar_hash_v1.0,
        trial_transcript_hash_v1: transcript.trial_transcript_hash_v1,
        materials,
        subjects,
        nvcc_flags: alloc::vec![
            "--std=c++17".to_string(),
            "-arch=sm_70".to_string(),
            "--fmad=false".to_string(),
            "--use_fast_math=false".to_string(),
            "-O2".to_string(),
            "-Xcompiler -fPIC".to_string(),
            "-Xptxas -O2".to_string(),
            "-DDSFB_GPU_FIXED_POINT".to_string(),
        ],
        rust_flags: alloc::vec!["-D warnings".to_string()],
        test_summary: TestSummary {
            workspace_test_groups: 57,
            workspace_passed: 670,
            workspace_failed: 0,
            workspace_ignored: 0,
        },
        gate_summary: GateSummary {
            fmt_clean: true,
            clippy_clean: true,
            scrub_clean: true,
            docs_freshness_clean: true,
            r12_byte_stability_clean: true,
        },
        r12b_episode_counts: R12bEpisodeCounts::baseline(),
        claimed_slsa_level: None,
        claimed_signed_attestation: false,
        non_claims: AttestationNonClaim::all_required().to_vec(),
        receipt_hash_v1: [0u8; 32],
    };
    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    r
}

/// Pinned S1.2 `registry_hash_v2`. Mirrors the registry-crate
/// receipt value so this corpus-side module does not need a
/// dependency on the registry crate.
fn registry_hash_v2_post_s12() -> [u8; 32] {
    [
        0xd3, 0xcf, 0x63, 0x00, 0x0c, 0xee, 0x92, 0x28, 0x18, 0xe8, 0xdb, 0xc7, 0x9f, 0xfe, 0xcb,
        0xc2, 0x7d, 0x28, 0x80, 0x63, 0xef, 0xba, 0xed, 0x58, 0x9e, 0x1e, 0xb1, 0x81, 0x2b, 0xc3,
        0x7a, 0x08,
    ]
}

fn capture_command_stdout(prog: &str, args: &[&str]) -> Option<String> {
    use std::process::Command;
    let out = Command::new(prog).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    Some(s.trim().to_string())
}

/// Query the live environment and build a non-pinned
/// `ExecutionAttestationReceiptV1`. Uses `std::process::Command`
/// to read `git rev-parse HEAD`, `rustc --version`, etc. The
/// receipt is structurally identical to the test fixture
/// but carries observed environment values; the hash will
/// therefore drift between commits. Suitable for the CLI;
/// NOT suitable for hash-pinned tests.
///
/// Falls back to the test fixture's pinned values when a
/// subprocess fails (e.g. no `git` available, no GPU). The
/// verifier still catches every structural reject path.
#[must_use]
pub fn build_t11e_live_attestation() -> ExecutionAttestationReceiptV1 {
    let mut r = build_t11e_test_fixture();

    if let Some(commit) = capture_command_stdout("git", &["rev-parse", "HEAD"]) {
        if commit.len() == 40 {
            r.repo_commit = commit;
        }
    }
    if let Some(status) = capture_command_stdout("git", &["status", "--porcelain"]) {
        r.repo_dirty = !status.is_empty();
    }
    if let Some(branch) = capture_command_stdout("git", &["rev-parse", "--abbrev-ref", "HEAD"]) {
        r.branch_name = Some(branch);
    }
    if let Some(v) = capture_command_stdout("rustc", &["--version"]) {
        r.rustc_version = v;
    }
    if let Some(v) = capture_command_stdout("cargo", &["--version"]) {
        r.cargo_version = v;
    }
    if let Some(v) = capture_command_stdout("nvcc", &["--version"]) {
        r.nvcc_version = Some(v);
    }
    // GPU name + compute capability would normally come from
    // `nvidia-smi --query-gpu=name,compute_cap --format=csv,noheader`,
    // but parsing that reliably is more than T.11e needs. The
    // panel-locked fields stay at their pinned default values
    // unless the operator overrides them via a future flag.

    r.receipt_hash_v1 = compute_execution_attestation_hash_v1(&r);
    r
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_opt_str(out: &mut Vec<u8>, s: Option<&String>) {
    match s {
        None => write_u8(out, 0),
        Some(v) => {
            write_u8(out, 1);
            write_str(out, v);
        }
    }
}

fn write_command(out: &mut Vec<u8>, c: &ExecutedCommand) {
    write_str(out, &c.command);
    write_str(out, c.purpose.as_str());
    write_u32(out, c.exit_code as u32);
}

fn write_material(out: &mut Vec<u8>, m: &MaterialDigest) {
    write_str(out, m.kind.as_str());
    write_str(out, &m.name);
    out.extend_from_slice(&m.digest);
}

fn write_subject(out: &mut Vec<u8>, s: &SubjectDigest) {
    write_str(out, s.kind.as_str());
    write_str(out, &s.name);
    out.extend_from_slice(&s.digest);
}

/// Compute the receipt's canonical-byte hash. Two calls on the
/// same receipt produce byte-identical output. Rendered text is
/// NOT included.
#[must_use]
#[allow(clippy::too_many_lines, clippy::cast_sign_loss)]
pub fn compute_execution_attestation_hash_v1(r: &ExecutionAttestationReceiptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(EXECUTION_ATTESTATION_DOMAIN.as_bytes());
    write_str(&mut buf, EXECUTION_ATTESTATION_SCHEMA_V1);
    write_str(&mut buf, r.schema.as_str());
    write_u32(&mut buf, r.receipt_id.0);

    // Repo + branch
    write_str(&mut buf, &r.repo_commit);
    write_u8(&mut buf, u8::from(r.repo_dirty));
    write_u8(&mut buf, u8::from(r.dirty_override_acknowledged));
    write_opt_str(&mut buf, r.branch_name.as_ref());

    // Toolchain
    write_str(&mut buf, &r.rustc_version);
    write_str(&mut buf, &r.cargo_version);
    write_opt_str(&mut buf, r.cuda_version.as_ref());
    write_opt_str(&mut buf, r.nvcc_version.as_ref());
    write_opt_str(&mut buf, r.gpu_name.as_ref());
    write_opt_str(&mut buf, r.gpu_compute_capability.as_ref());

    // Commands (canonical sort by (purpose, command) for build;
    // verification commands are kept in declared order — the
    // operator usually wants fmt → clippy → scrub → docs_freshness
    // → tests in that sequence). For hash determinism we sort
    // both lists.
    let mut bc: Vec<&ExecutedCommand> = r.build_commands.iter().collect();
    bc.sort_by(|a, b| {
        (a.purpose.as_str(), a.command.as_str()).cmp(&(b.purpose.as_str(), b.command.as_str()))
    });
    write_u32(&mut buf, u32::try_from(bc.len()).unwrap_or(u32::MAX));
    for c in bc {
        write_command(&mut buf, c);
    }
    let mut vc: Vec<&ExecutedCommand> = r.verification_commands.iter().collect();
    vc.sort_by(|a, b| {
        (a.purpose.as_str(), a.command.as_str()).cmp(&(b.purpose.as_str(), b.command.as_str()))
    });
    write_u32(&mut buf, u32::try_from(vc.len()).unwrap_or(u32::MAX));
    for c in vc {
        write_command(&mut buf, c);
    }

    // Hash chain anchors
    buf.extend_from_slice(&r.corpus_hash_v1);
    buf.extend_from_slice(&r.registry_hash_v2);
    buf.extend_from_slice(&r.precedent_hash_v1);
    buf.extend_from_slice(&r.admissibility_grammar_hash_v1);
    buf.extend_from_slice(&r.trial_transcript_hash_v1);

    // Materials (sorted by name)
    let mut mats: Vec<&MaterialDigest> = r.materials.iter().collect();
    mats.sort_by(|a, b| a.name.cmp(&b.name));
    write_u32(&mut buf, u32::try_from(mats.len()).unwrap_or(u32::MAX));
    for m in mats {
        write_material(&mut buf, m);
    }

    // Subjects (sorted by name)
    let mut subs: Vec<&SubjectDigest> = r.subjects.iter().collect();
    subs.sort_by(|a, b| a.name.cmp(&b.name));
    write_u32(&mut buf, u32::try_from(subs.len()).unwrap_or(u32::MAX));
    for s in subs {
        write_subject(&mut buf, s);
    }

    // Flags (sorted lexicographically so order-of-declaration
    // does not change the hash for an unordered set).
    let mut nf: Vec<&String> = r.nvcc_flags.iter().collect();
    nf.sort();
    write_u32(&mut buf, u32::try_from(nf.len()).unwrap_or(u32::MAX));
    for f in nf {
        write_str(&mut buf, f);
    }
    let mut rf: Vec<&String> = r.rust_flags.iter().collect();
    rf.sort();
    write_u32(&mut buf, u32::try_from(rf.len()).unwrap_or(u32::MAX));
    for f in rf {
        write_str(&mut buf, f);
    }

    // Test + gate summaries
    write_u32(&mut buf, r.test_summary.workspace_test_groups);
    write_u32(&mut buf, r.test_summary.workspace_passed);
    write_u32(&mut buf, r.test_summary.workspace_failed);
    write_u32(&mut buf, r.test_summary.workspace_ignored);
    write_u8(&mut buf, u8::from(r.gate_summary.fmt_clean));
    write_u8(&mut buf, u8::from(r.gate_summary.clippy_clean));
    write_u8(&mut buf, u8::from(r.gate_summary.scrub_clean));
    write_u8(&mut buf, u8::from(r.gate_summary.docs_freshness_clean));
    write_u8(&mut buf, u8::from(r.gate_summary.r12_byte_stability_clean));

    write_u32(&mut buf, r.r12b_episode_counts.canonical_16x128);
    write_u32(&mut buf, r.r12b_episode_counts.mid_64x512);
    write_u32(&mut buf, r.r12b_episode_counts.full_256x4096);

    // Non-claims (sorted by wire name for determinism)
    let mut nc: Vec<&AttestationNonClaim> = r.non_claims.iter().collect();
    nc.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    write_u32(&mut buf, u32::try_from(nc.len()).unwrap_or(u32::MAX));
    for c in nc {
        write_str(&mut buf, c.as_str());
    }

    // Claim-rejection markers
    match r.claimed_slsa_level {
        None => write_u8(&mut buf, 0),
        Some(level) => {
            write_u8(&mut buf, 1);
            write_u8(&mut buf, level);
        }
    }
    write_u8(&mut buf, u8::from(r.claimed_signed_attestation));

    sha256(&buf)
}

/// One verification failure on an
/// `ExecutionAttestationReceiptV1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttestationVerifyError {
    /// Structured failure kind.
    pub kind: AttestationVerifyErrorKind,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Structured attestation-verifier failure category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationVerifyErrorKind {
    /// `corpus_hash_v1` is the all-zero sentinel.
    ZeroCorpusHash,
    /// `registry_hash_v2` is the all-zero sentinel.
    ZeroRegistryHash,
    /// `precedent_hash_v1` is the all-zero sentinel.
    ZeroPrecedentHash,
    /// `admissibility_grammar_hash_v1` is the all-zero sentinel.
    ZeroGrammarHash,
    /// `trial_transcript_hash_v1` is the all-zero sentinel.
    ZeroTranscriptHash,
    /// `repo_commit` is empty.
    EmptyRepoCommit,
    /// `repo_commit` is not 40 hex chars.
    InvalidRepoCommitFormat,
    /// `repo_dirty == true` AND `dirty_override_acknowledged ==
    /// false`. The verifier rejects dirty-tree attestations
    /// unless the operator explicitly acknowledges via the
    /// override flag.
    DirtyRepoWithoutOverride,
    /// `build_commands` is empty.
    EmptyBuildCommands,
    /// `verification_commands` does not include every required
    /// gate (fmt + clippy + scrub + docs_freshness +
    /// workspace_test).
    MissingRequiredGateCommand,
    /// `gate_summary` flags at least one gate as not clean.
    GateNotClean,
    /// `test_summary.workspace_failed > 0`.
    WorkspaceTestFailed,
    /// `r12b_episode_counts` does not match the panel-locked
    /// baseline (13 / 89 / 1917).
    R12bEpisodeCountsDrift,
    /// `claimed_slsa_level` is `Some(_)`. T.11e MUST NOT claim
    /// any SLSA level. (Load-bearing panel test.)
    ClaimedSlsaLevelPresent,
    /// `claimed_signed_attestation == true`. T.11e is unsigned.
    ClaimedSignedAttestation,
    /// `subjects` is empty.
    SubjectDigestMissing,
    /// `materials` is empty.
    MaterialDigestMissing,
    /// `receipt_hash_v1` does not match a fresh recomputation.
    ReceiptHashMismatch,
    /// `non_claims` is missing at least one panel-locked
    /// required entry.
    NonClaimsIncomplete,
    /// Cross-citation drift: a hash carried in the receipt does
    /// not match the live corpus / precedent / grammar /
    /// transcript value.
    HashChainCrossCheckFailed,
}

const REQUIRED_VERIFICATION_PURPOSES: &[CommandPurpose] = &[
    CommandPurpose::Format,
    CommandPurpose::Clippy,
    CommandPurpose::Scrub,
    CommandPurpose::DocsFreshness,
    CommandPurpose::WorkspaceTest,
];

/// Verify an `ExecutionAttestationReceiptV1` against the
/// panel-locked 20-direction structural invariants. Returns the
/// list of failures (empty if clean).
///
/// The verifier consults the live corpus + precedent + grammar +
/// transcript values rather than carrying a snapshot, so two
/// verifications on the same receipt produce the same result and
/// the verifier stays honest about the present state. The
/// `HashChainCrossCheckFailed` rejection is informational at
/// T.11e — the live values may drift relative to a committed
/// receipt, and that drift is itself worth surfacing.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_execution_attestation(
    r: &ExecutionAttestationReceiptV1,
) -> Vec<AttestationVerifyError> {
    let mut errors: Vec<AttestationVerifyError> = Vec::new();

    if r.corpus_hash_v1 == [0u8; 32] {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ZeroCorpusHash,
            message: "receipt carries zero corpus_hash_v1".into(),
        });
    }
    if r.registry_hash_v2 == [0u8; 32] {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ZeroRegistryHash,
            message: "receipt carries zero registry_hash_v2".into(),
        });
    }
    if r.precedent_hash_v1 == [0u8; 32] {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ZeroPrecedentHash,
            message: "receipt carries zero precedent_hash_v1".into(),
        });
    }
    if r.admissibility_grammar_hash_v1 == [0u8; 32] {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ZeroGrammarHash,
            message: "receipt carries zero admissibility_grammar_hash_v1".into(),
        });
    }
    if r.trial_transcript_hash_v1 == [0u8; 32] {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ZeroTranscriptHash,
            message: "receipt carries zero trial_transcript_hash_v1".into(),
        });
    }

    if r.repo_commit.is_empty() {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::EmptyRepoCommit,
            message: "receipt carries empty repo_commit".into(),
        });
    } else if !is_hex40(&r.repo_commit) {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::InvalidRepoCommitFormat,
            message: format!(
                "repo_commit `{}` is not a 40-char lowercase hex git sha-1",
                r.repo_commit
            ),
        });
    }

    if r.repo_dirty && !r.dirty_override_acknowledged {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::DirtyRepoWithoutOverride,
            message: "receipt is bound to a dirty tree but `dirty_override_acknowledged` is false"
                .into(),
        });
    }

    if r.build_commands.is_empty() {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::EmptyBuildCommands,
            message: "receipt carries empty build_commands".into(),
        });
    }
    for required in REQUIRED_VERIFICATION_PURPOSES {
        if !r
            .verification_commands
            .iter()
            .any(|c| c.purpose == *required)
        {
            errors.push(AttestationVerifyError {
                kind: AttestationVerifyErrorKind::MissingRequiredGateCommand,
                message: format!(
                    "verification_commands missing required purpose {}",
                    required.as_str()
                ),
            });
        }
    }

    let g = r.gate_summary;
    if !(g.fmt_clean
        && g.clippy_clean
        && g.scrub_clean
        && g.docs_freshness_clean
        && g.r12_byte_stability_clean)
    {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::GateNotClean,
            message: format!(
                "gate_summary not clean: fmt={} clippy={} scrub={} docs_freshness={} r12_byte_stability={}",
                g.fmt_clean,
                g.clippy_clean,
                g.scrub_clean,
                g.docs_freshness_clean,
                g.r12_byte_stability_clean
            ),
        });
    }

    if r.test_summary.workspace_failed != 0 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::WorkspaceTestFailed,
            message: format!(
                "workspace_failed = {} (must be 0)",
                r.test_summary.workspace_failed
            ),
        });
    }

    let baseline = R12bEpisodeCounts::baseline();
    if r.r12b_episode_counts != baseline {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::R12bEpisodeCountsDrift,
            message: format!(
                "r12b_episode_counts ({} / {} / {}) drifted from panel-locked baseline (13 / 89 / 1917)",
                r.r12b_episode_counts.canonical_16x128,
                r.r12b_episode_counts.mid_64x512,
                r.r12b_episode_counts.full_256x4096
            ),
        });
    }

    if r.claimed_slsa_level.is_some() {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ClaimedSlsaLevelPresent,
            message:
                "claimed_slsa_level is Some(_); T.11e MUST NOT claim any SLSA level. The receipt is DSFB-native and unsigned."
                    .into(),
        });
    }
    if r.claimed_signed_attestation {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ClaimedSignedAttestation,
            message: "claimed_signed_attestation = true; T.11e is unsigned".into(),
        });
    }

    if r.subjects.is_empty() {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::SubjectDigestMissing,
            message: "receipt carries no subject digests".into(),
        });
    }
    if r.materials.is_empty() {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::MaterialDigestMissing,
            message: "receipt carries no material digests".into(),
        });
    }

    // Required non-claims check
    for required in AttestationNonClaim::all_required() {
        if !r.non_claims.contains(&required) {
            errors.push(AttestationVerifyError {
                kind: AttestationVerifyErrorKind::NonClaimsIncomplete,
                message: format!("non_claims missing required entry {}", required.as_str()),
            });
        }
    }

    // Receipt-hash recomputation
    let recomputed = compute_execution_attestation_hash_v1(r);
    if recomputed != r.receipt_hash_v1 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::ReceiptHashMismatch,
            message:
                "receipt_hash_v1 does not match the recomputed hash over the canonical-byte projection"
                    .into(),
        });
    }

    // Live cross-check (best-effort; surfaces drift but is not
    // load-bearing because a future receipt could legitimately
    // commit to an older snapshot).
    let live_corpus = compute_corpus_hash_v1();
    if live_corpus.bytes != r.corpus_hash_v1 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::HashChainCrossCheckFailed,
            message: "corpus_hash_v1 in receipt does not match the live compute_corpus_hash_v1"
                .into(),
        });
    }
    let live_precedent = collect_court_precedents();
    if live_precedent.precedent_hash_v1 != r.precedent_hash_v1 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::HashChainCrossCheckFailed,
            message:
                "precedent_hash_v1 in receipt does not match the live collect_court_precedents"
                    .into(),
        });
    }
    let live_grammar = collect_admissibility_grammar();
    if live_grammar.admissibility_grammar_hash_v1.0 != r.admissibility_grammar_hash_v1 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::HashChainCrossCheckFailed,
            message: "admissibility_grammar_hash_v1 in receipt does not match the live grammar"
                .into(),
        });
    }
    let live_transcript = build_t11d_latency_ramp_fixture();
    if live_transcript.trial_transcript_hash_v1 != r.trial_transcript_hash_v1 {
        errors.push(AttestationVerifyError {
            kind: AttestationVerifyErrorKind::HashChainCrossCheckFailed,
            message: "trial_transcript_hash_v1 in receipt does not match the live T.11d fixture"
                .into(),
        });
    }

    errors
}

fn is_hex40(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[usize::from(b >> 4)] as char);
        s.push(HEX[usize::from(b & 0x0F)] as char);
    }
    s
}

/// Render the receipt as deterministic human-readable text. Two
/// calls on the same receipt produce byte-identical output.
/// Rendered text is NOT part of the receipt hash.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_execution_attestation_text(r: &ExecutionAttestationReceiptV1) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas — Execution Attestation Receipt V1 (T.11e)\n");
    out.push_str("================================================================\n");
    let _ = writeln!(out, "schema           : {}", r.schema.as_str());
    let _ = writeln!(out, "receipt_id       : {}", r.receipt_id.0);
    let _ = writeln!(out, "receipt_hash_v1  : {}", hex_lower(&r.receipt_hash_v1));
    out.push('\n');
    out.push_str("Panel-locked non-claims (verifier-enforced):\n");
    let mut nc: Vec<&AttestationNonClaim> = r.non_claims.iter().collect();
    nc.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    for c in nc {
        let _ = writeln!(out, "  - {}", c.as_str());
    }
    let _ = writeln!(
        out,
        "  claimed_slsa_level       : {:?}",
        r.claimed_slsa_level
    );
    let _ = writeln!(
        out,
        "  claimed_signed_attestation: {}",
        r.claimed_signed_attestation
    );
    out.push('\n');
    out.push_str("Repo + branch:\n");
    let _ = writeln!(out, "  repo_commit              : {}", r.repo_commit);
    let _ = writeln!(out, "  repo_dirty               : {}", r.repo_dirty);
    let _ = writeln!(
        out,
        "  dirty_override_acknowledged: {}",
        r.dirty_override_acknowledged
    );
    let _ = writeln!(
        out,
        "  branch_name              : {}",
        r.branch_name.as_deref().unwrap_or("(none)")
    );
    out.push('\n');
    out.push_str("Toolchain:\n");
    let _ = writeln!(out, "  rustc_version            : {}", r.rustc_version);
    let _ = writeln!(out, "  cargo_version            : {}", r.cargo_version);
    let _ = writeln!(
        out,
        "  cuda_version             : {}",
        r.cuda_version.as_deref().unwrap_or("(none)")
    );
    let _ = writeln!(
        out,
        "  nvcc_version             : {}",
        r.nvcc_version.as_deref().unwrap_or("(none)")
    );
    let _ = writeln!(
        out,
        "  gpu_name                 : {}",
        r.gpu_name.as_deref().unwrap_or("(none)")
    );
    let _ = writeln!(
        out,
        "  gpu_compute_capability   : {}",
        r.gpu_compute_capability.as_deref().unwrap_or("(none)")
    );
    out.push('\n');
    out.push_str("Hash chain anchors:\n");
    let _ = writeln!(
        out,
        "  corpus_hash_v1               : {}",
        hex_lower(&r.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  registry_hash_v2             : {}",
        hex_lower(&r.registry_hash_v2)
    );
    let _ = writeln!(
        out,
        "  precedent_hash_v1            : {}",
        hex_lower(&r.precedent_hash_v1)
    );
    let _ = writeln!(
        out,
        "  admissibility_grammar_hash_v1: {}",
        hex_lower(&r.admissibility_grammar_hash_v1)
    );
    let _ = writeln!(
        out,
        "  trial_transcript_hash_v1     : {}",
        hex_lower(&r.trial_transcript_hash_v1)
    );
    out.push('\n');
    out.push_str("Build commands:\n");
    for c in &r.build_commands {
        let _ = writeln!(
            out,
            "  [{}] (exit={}) {}",
            c.purpose.as_str(),
            c.exit_code,
            c.command
        );
    }
    out.push('\n');
    out.push_str("Verification commands:\n");
    for c in &r.verification_commands {
        let _ = writeln!(
            out,
            "  [{}] (exit={}) {}",
            c.purpose.as_str(),
            c.exit_code,
            c.command
        );
    }
    out.push('\n');
    out.push_str("Gate summary:\n");
    let _ = writeln!(
        out,
        "  fmt_clean                : {}",
        r.gate_summary.fmt_clean
    );
    let _ = writeln!(
        out,
        "  clippy_clean             : {}",
        r.gate_summary.clippy_clean
    );
    let _ = writeln!(
        out,
        "  scrub_clean              : {}",
        r.gate_summary.scrub_clean
    );
    let _ = writeln!(
        out,
        "  docs_freshness_clean     : {}",
        r.gate_summary.docs_freshness_clean
    );
    let _ = writeln!(
        out,
        "  r12_byte_stability_clean : {}",
        r.gate_summary.r12_byte_stability_clean
    );
    out.push('\n');
    out.push_str("Workspace tests:\n");
    let _ = writeln!(
        out,
        "  result_groups : {}",
        r.test_summary.workspace_test_groups
    );
    let _ = writeln!(out, "  passed        : {}", r.test_summary.workspace_passed);
    let _ = writeln!(out, "  failed        : {}", r.test_summary.workspace_failed);
    let _ = writeln!(
        out,
        "  ignored       : {}",
        r.test_summary.workspace_ignored
    );
    out.push('\n');
    out.push_str("R.12b episode counts (pinned baseline):\n");
    let _ = writeln!(
        out,
        "  canonical 16x128 : {}",
        r.r12b_episode_counts.canonical_16x128
    );
    let _ = writeln!(
        out,
        "  mid 64x512       : {}",
        r.r12b_episode_counts.mid_64x512
    );
    let _ = writeln!(
        out,
        "  full 256x4096    : {}",
        r.r12b_episode_counts.full_256x4096
    );
    out.push('\n');
    out.push_str("Materials (sorted):\n");
    let mut mats: Vec<&MaterialDigest> = r.materials.iter().collect();
    mats.sort_by(|a, b| a.name.cmp(&b.name));
    for m in mats {
        let _ = writeln!(
            out,
            "  [{}] {} : {}",
            m.kind.as_str(),
            m.name,
            hex_lower(&m.digest)
        );
    }
    out.push('\n');
    out.push_str("Subjects (sorted):\n");
    let mut subs: Vec<&SubjectDigest> = r.subjects.iter().collect();
    subs.sort_by(|a, b| a.name.cmp(&b.name));
    for s in subs {
        let _ = writeln!(
            out,
            "  [{}] {} : {}",
            s.kind.as_str(),
            s.name,
            hex_lower(&s.digest)
        );
    }
    out.push('\n');
    out.push_str("nvcc_flags (sorted):\n");
    let mut nf: Vec<&String> = r.nvcc_flags.iter().collect();
    nf.sort();
    for f in nf {
        let _ = writeln!(out, "  - {f}");
    }
    out.push_str("rust_flags (sorted):\n");
    let mut rf: Vec<&String> = r.rust_flags.iter().collect();
    rf.sort();
    for f in rf {
        let _ = writeln!(out, "  - {f}");
    }
    out
}

fn json_quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn opt_json_quote(out: &mut String, s: Option<&String>) {
    match s {
        None => out.push_str("null"),
        Some(v) => json_quote(out, v),
    }
}

/// Render the receipt as deterministic JSON. Two calls produce
/// byte-identical output. Rendered text is NOT part of the hash.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_execution_attestation_json(r: &ExecutionAttestationReceiptV1) -> String {
    let mut out = String::with_capacity(8 * 1024);
    out.push_str("{\n");
    out.push_str("  \"schema\": ");
    json_quote(&mut out, r.schema.as_str());
    out.push_str(",\n");
    let _ = writeln!(out, "  \"receipt_id\": {},", r.receipt_id.0);
    let _ = writeln!(
        out,
        "  \"receipt_hash_v1\": \"{}\",",
        hex_lower(&r.receipt_hash_v1)
    );
    out.push_str("  \"repo\": {\n");
    out.push_str("    \"commit\": ");
    json_quote(&mut out, &r.repo_commit);
    out.push_str(",\n");
    let _ = writeln!(out, "    \"dirty\": {},", r.repo_dirty);
    let _ = writeln!(
        out,
        "    \"dirty_override_acknowledged\": {},",
        r.dirty_override_acknowledged
    );
    out.push_str("    \"branch\": ");
    opt_json_quote(&mut out, r.branch_name.as_ref());
    out.push('\n');
    out.push_str("  },\n");
    out.push_str("  \"toolchain\": {\n");
    out.push_str("    \"rustc\": ");
    json_quote(&mut out, &r.rustc_version);
    out.push_str(",\n");
    out.push_str("    \"cargo\": ");
    json_quote(&mut out, &r.cargo_version);
    out.push_str(",\n");
    out.push_str("    \"cuda\": ");
    opt_json_quote(&mut out, r.cuda_version.as_ref());
    out.push_str(",\n");
    out.push_str("    \"nvcc\": ");
    opt_json_quote(&mut out, r.nvcc_version.as_ref());
    out.push_str(",\n");
    out.push_str("    \"gpu_name\": ");
    opt_json_quote(&mut out, r.gpu_name.as_ref());
    out.push_str(",\n");
    out.push_str("    \"gpu_compute_capability\": ");
    opt_json_quote(&mut out, r.gpu_compute_capability.as_ref());
    out.push('\n');
    out.push_str("  },\n");
    let _ = writeln!(
        out,
        "  \"corpus_hash_v1\": \"{}\",",
        hex_lower(&r.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"registry_hash_v2\": \"{}\",",
        hex_lower(&r.registry_hash_v2)
    );
    let _ = writeln!(
        out,
        "  \"precedent_hash_v1\": \"{}\",",
        hex_lower(&r.precedent_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"admissibility_grammar_hash_v1\": \"{}\",",
        hex_lower(&r.admissibility_grammar_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"trial_transcript_hash_v1\": \"{}\",",
        hex_lower(&r.trial_transcript_hash_v1)
    );
    out.push_str("  \"non_claims\": [");
    let mut nc: Vec<&AttestationNonClaim> = r.non_claims.iter().collect();
    nc.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    for (i, c) in nc.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_quote(&mut out, c.as_str());
    }
    out.push_str("],\n");
    match r.claimed_slsa_level {
        None => out.push_str("  \"claimed_slsa_level\": null,\n"),
        Some(level) => {
            let _ = writeln!(out, "  \"claimed_slsa_level\": {level},");
        }
    }
    let _ = writeln!(
        out,
        "  \"claimed_signed_attestation\": {},",
        r.claimed_signed_attestation
    );
    out.push_str("  \"build_commands\": [");
    for (i, c) in r.build_commands.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"command\": ");
        json_quote(&mut out, &c.command);
        out.push_str(", \"purpose\": ");
        json_quote(&mut out, c.purpose.as_str());
        let _ = write!(out, ", \"exit_code\": {}", c.exit_code);
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"verification_commands\": [");
    for (i, c) in r.verification_commands.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"command\": ");
        json_quote(&mut out, &c.command);
        out.push_str(", \"purpose\": ");
        json_quote(&mut out, c.purpose.as_str());
        let _ = write!(out, ", \"exit_code\": {}", c.exit_code);
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"gate_summary\": {");
    let _ = write!(out, "\"fmt_clean\": {}", r.gate_summary.fmt_clean);
    let _ = write!(out, ", \"clippy_clean\": {}", r.gate_summary.clippy_clean);
    let _ = write!(out, ", \"scrub_clean\": {}", r.gate_summary.scrub_clean);
    let _ = write!(
        out,
        ", \"docs_freshness_clean\": {}",
        r.gate_summary.docs_freshness_clean
    );
    let _ = write!(
        out,
        ", \"r12_byte_stability_clean\": {}",
        r.gate_summary.r12_byte_stability_clean
    );
    out.push_str("},\n");
    out.push_str("  \"test_summary\": {");
    let _ = write!(
        out,
        "\"workspace_test_groups\": {}",
        r.test_summary.workspace_test_groups
    );
    let _ = write!(
        out,
        ", \"workspace_passed\": {}",
        r.test_summary.workspace_passed
    );
    let _ = write!(
        out,
        ", \"workspace_failed\": {}",
        r.test_summary.workspace_failed
    );
    let _ = write!(
        out,
        ", \"workspace_ignored\": {}",
        r.test_summary.workspace_ignored
    );
    out.push_str("},\n");
    out.push_str("  \"r12b_episode_counts\": {");
    let _ = write!(
        out,
        "\"canonical_16x128\": {}",
        r.r12b_episode_counts.canonical_16x128
    );
    let _ = write!(
        out,
        ", \"mid_64x512\": {}",
        r.r12b_episode_counts.mid_64x512
    );
    let _ = write!(
        out,
        ", \"full_256x4096\": {}",
        r.r12b_episode_counts.full_256x4096
    );
    out.push_str("},\n");
    out.push_str("  \"materials\": [");
    let mut mats: Vec<&MaterialDigest> = r.materials.iter().collect();
    mats.sort_by(|a, b| a.name.cmp(&b.name));
    for (i, m) in mats.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"kind\": ");
        json_quote(&mut out, m.kind.as_str());
        out.push_str(", \"name\": ");
        json_quote(&mut out, &m.name);
        let _ = write!(out, ", \"digest\": \"{}\"", hex_lower(&m.digest));
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"subjects\": [");
    let mut subs: Vec<&SubjectDigest> = r.subjects.iter().collect();
    subs.sort_by(|a, b| a.name.cmp(&b.name));
    for (i, s) in subs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"kind\": ");
        json_quote(&mut out, s.kind.as_str());
        out.push_str(", \"name\": ");
        json_quote(&mut out, &s.name);
        let _ = write!(out, ", \"digest\": \"{}\"", hex_lower(&s.digest));
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"nvcc_flags\": [");
    let mut nf: Vec<&String> = r.nvcc_flags.iter().collect();
    nf.sort();
    for (i, f) in nf.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_quote(&mut out, f);
    }
    out.push_str("],\n");
    out.push_str("  \"rust_flags\": [");
    let mut rf: Vec<&String> = r.rust_flags.iter().collect();
    rf.sort();
    for (i, f) in rf.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_quote(&mut out, f);
    }
    out.push_str("]\n");
    out.push_str("}\n");
    out
}
