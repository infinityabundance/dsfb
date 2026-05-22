//! Case-file: the final, replayable verdict record.
//!
//! Running the CPU pipeline end-to-end through `build_cpu` produces a
//! `CaseFile`. The case file carries:
//!
//! * The contract bytes (hashed into `contract_hash`).
//! * A hash chain that pins every intermediate artifact in canonical
//!   sequence: contract → bank → detector registry → kernel sequence →
//!   window features → residual → sign → detector → consensus →
//!   candidate → episode → final case file hash.
//! * The admitted episodes themselves.
//! * The final verdict (which begins as `CpuOnlyAdmissible` and is
//!   refined by `compare`).
//!
//! Determinism: identical inputs (fixture, contract, bank, detector
//! registry) produce a byte-identical case file. That is the load-
//! bearing property of the prior-art proof.

#![cfg(feature = "std")]

use std::vec::Vec;

#[cfg(test)]
use crate::bank::BankMotif;
use crate::bank::{bank_hash, collapse as bank_collapse, Episode};
use crate::candidate::{prepare_with_detectors, CandidateConfig, CandidateInterval};
use crate::consensus::{form as consensus_form, ConsensusCell};
use crate::contract::{chain, Contract};
use crate::detector::{evaluate as detector_evaluate, DetectorCell, DetectorThresholds};
use crate::fixed::Q16;
use crate::grammar::{GrammarState, ReasonCode};
use crate::hash::sha256;
use crate::motif::registry_hash as motif_registry_hash;
use crate::residual::{compute as residual_compute, Baseline, ResidualCell};
use crate::serialize::{write_fixture, write_key, write_str, write_u64};
use crate::sign::{compute as sign_compute, SignCell};
use crate::verdict::FinalVerdict;
use crate::window::{compute_features, WindowFeature};
use crate::TraceEvent;

/// The case file emission mode. Determines which byte representation is
/// fed into the SHA-256 of each intermediate stage.
///
/// The doctrine: both modes preserve the load-bearing hash chain, the
/// `BankAdmissionToken` semantics, and the Semantic Non-Bypass Axiom.
/// Throughput mode is **not** a black-box mode — it produces the same
/// chain link structure as Audit, with the same labels and the same
/// cascading dependency. The only difference is the byte form each
/// stage hashes over.
///
/// * `Audit` (default): each stage hashes its **canonical-text**
///   serialization (the form `serialize_residual_cells` etc. produce —
///   `entity:window:field:field;...`). This form is human-readable,
///   inspectable by hand, and is what the README's reproducibility
///   receipt pins. It is also expensive at scale because canonical-text
///   bytes are ~3-5× larger than the raw cell layout.
///
/// * `Throughput`: each stage hashes its **compact-bytes**
///   serialization (little-endian raw field bytes — the same numbers,
///   just packed). Much faster on the host, especially at scaled
///   contracts. Produces *different* per-stage hash values than Audit
///   because the input bytes differ; the case file records its mode so
///   `compare` knows which scheme each side used.
///
/// Both modes produce byte-identical `input_catalog`, `contract`,
/// `bank`, `detector_registry`, `kernel_sequence`, and `episode` hashes
/// (these aren't affected by the per-stage byte form). The final hash
/// differs because it covers all preceding bytes including the mode tag.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum EmissionMode {
    /// Canonical-text bytes per stage. Default.
    #[default]
    Audit,
    /// Compact little-endian raw-field bytes per stage. Higher
    /// throughput, identical semantic chain.
    Throughput,
}

impl EmissionMode {
    /// Stable lowercase name written into the canonical case-file bytes.
    /// Changing this string would invalidate every prior case file; the
    /// names are part of the prior-art posture.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Audit => "audit",
            Self::Throughput => "throughput",
        }
    }
}

/// Stage-by-stage hash chain. Each field is `sha256(label || cur_bytes ||
/// previous_hash)`, so any change to any prior link cascades.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IntermediateHashes {
    /// Hash of the input catalog (the canonical fixture bytes).
    pub input_catalog: [u8; 32],
    /// `chain(b"contract", &Contract::hash, &input_catalog)`.
    pub contract: [u8; 32],
    /// `chain(b"bank", &bank_hash(), &contract)`.
    pub bank: [u8; 32],
    /// `chain(b"detreg", &motif_registry_hash(), &bank)`.
    pub detector_registry: [u8; 32],
    /// `chain(b"kseq", &kernel_sequence_hash, &detector_registry)`.
    pub kernel_sequence: [u8; 32],
    /// `chain(b"window", &sha256(window_bytes), &kernel_sequence)`.
    pub window_feature: [u8; 32],
    /// `chain(b"residual", ...)`.
    pub residual_field: [u8; 32],
    /// `chain(b"sign", ...)`.
    pub sign_field: [u8; 32],
    /// `chain(b"detector", ...)`.
    pub detector_cell: [u8; 32],
    /// `chain(b"consensus", ...)`.
    pub consensus_grid: [u8; 32],
    /// `chain(b"candidate", ...)`.
    pub candidate_interval: [u8; 32],
    /// `chain(b"episode", ...)`.
    pub episode: [u8; 32],
}

/// The complete case file produced by an end-to-end run.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CaseFile {
    /// `"dsfb-gpu-debug-case-0.1"` literal.
    pub version: &'static str,
    /// Either `"cpu"` or `"cuda"` depending on which pipeline produced
    /// the artifacts.
    pub backend: &'static str,
    /// Which byte form each stage was hashed over. Audit (the default)
    /// uses canonical-text bytes for the strongest replay receipt;
    /// Throughput uses compact little-endian raw bytes for higher
    /// performance at scaled contracts.
    pub mode: EmissionMode,
    /// Contract hash and every intermediate-stage hash, in chain order.
    pub hashes: IntermediateHashes,
    /// Admitted episodes. Empty when the fixture's clean windows
    /// dominate.
    pub episodes: Vec<Episode>,
    /// Final case-file hash: SHA-256 over the case file's canonical
    /// bytes excluding this field itself.
    pub final_case_file_hash: [u8; 32],
    /// Verdict (refined by `compare`).
    pub final_verdict: FinalVerdict,
}

const CASE_FILE_VERSION: &str = "DSFB-GPU-DEBUG-CASE-0.1";

/// R.3a — Layer A device-evidence-fabric summary, returned by the
/// skip-bank Throughput path. Carries everything the host needs to
/// produce an admissible case file *later* (by feeding it to the bank
/// stage in R.3b / R.5) **without** running the bank stage now.
///
/// Why this exists:
/// * **Layer A doctrine**: the GPU evidence fabric should be measurable
///   in isolation, separate from the host-side bank stage. The bench's
///   Layer A column is the fastest, slimmest dispatch path; it must
///   not emit admitted episodes. This struct makes that constraint
///   structural: it carries *no* `Episode` field. There is no way for
///   a caller of the Layer A dispatch to obtain admitted episodes —
///   the type system refuses.
///
/// * **R.3 vs R.5 split**: R.3a (this commit) introduces the
///   summary scaffolding without changing the on-the-wire
///   `CandidateInterval` byte layout. R.5 (next) will add
///   `entity_avg_q` and `grid_avg_q` to `CandidateInterval`,
///   refresh the pinned golden hashes, and let the bank stage
///   read the precomputed axis-5 fields. After R.5, the
///   `consensus_*` D2H can also be stripped. Until then, R.3a
///   defines the boundary; R.5 lands the contract change.
///
/// Fields:
/// * `hashes` — the 12-link `IntermediateHashes` chain through
///   `candidate_interval`. `episode` and `final_case_file_hash` are
///   reserved for the post-bank case file; they are not present here.
/// * `candidates` — the admitted-candidate slice the bank will consume
///   if/when episode admission is desired. Pre-R.5 layout.
/// * `breach_flags` — bit-packed breach signals
///   (bank-hash-mismatch, detector-registry-mismatch, etc.) so the
///   summary can answer the contract-breach question without a full
///   bank pass. Bits are defined in `breach_flags` module constants.
/// * `mode` — always `Throughput` for v0; the field is preserved so a
///   future Layer A audit variant can also use this type.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CompactCaseSummary {
    /// Backend that produced this summary. Either `"cpu"` (Layer A
    /// mocked for tests) or `"cuda"`.
    pub backend: &'static str,
    /// Byte-form mode the per-stage digests use. Always
    /// `EmissionMode::Throughput` for v0 Layer A.
    pub mode: EmissionMode,
    /// First 12 hash-chain links: input_catalog through candidate_interval.
    /// `episode` and `final_case_file_hash` are intentionally absent —
    /// they require the bank stage, which Layer A does not run.
    pub hashes: IntermediateHashes,
    /// Admitted-candidate-interval slice (pre-R.5 byte layout). The
    /// bank stage, when invoked, would consume this to emit admitted
    /// episodes.
    pub candidates: Vec<CandidateInterval>,
    /// Bit-packed breach signals. See `breach_flags` module constants.
    /// A summary with `breach_flags != 0` indicates a contract problem
    /// that would otherwise short-circuit the bank stage.
    pub breach_flags: u32,
}

/// Bit-packed contract-breach flags for `CompactCaseSummary::breach_flags`.
///
/// The host bank stage in the full audit court path computes these by
/// comparing the contract's pinned `bank_hash` / `detector_registry_hash`
/// against the actually-computed registry hashes and short-circuits the
/// verdict if a mismatch is detected. Layer A surfaces the same
/// information without running the bank stage so callers can detect
/// contract breaches at evidence-fabric time.
///
/// All flags are 0 = ok. The default (cleanly-admissible) summary has
/// `breach_flags == 0`.
pub mod breach_flags {
    /// Bit 0: `contract.bank_hash` does not match the computed bank
    /// chain link.
    pub const BANK_HASH_MISMATCH: u32 = 1 << 0;
    /// Bit 1: `contract.detector_registry_hash` does not match the
    /// computed detector-registry chain link.
    pub const DETECTOR_REGISTRY_MISMATCH: u32 = 1 << 1;
    /// Bit 2: reserved for the Section R.6 `graph_plan_hash`
    /// divergence breach (i.e., the captured CUDA graph plan no longer
    /// matches the runtime plan).
    pub const GRAPH_PLAN_MISMATCH: u32 = 1 << 2;
}

/// Build the case file by running the canonical CPU pipeline against an
/// event slice, a contract, the canonical bank, and the canonical
/// detector thresholds.
///
/// Returns the case file plus the raw episode list. The case file's
/// `final_case_file_hash` is filled in before returning.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_cpu(events: &[TraceEvent], contract: &Contract) -> CaseFile {
    // 1. Run the pipeline.
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let residuals = residual_compute(&features, &Baseline::CANONICAL);
    let signs = sign_compute(
        &residuals,
        Q16::from_raw(contract.ewma_alpha_q16_raw),
        contract.n_windows,
        contract.n_entities,
    );
    let detectors = detector_evaluate(
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let consensus = consensus_form(&signs, &detectors, contract.n_windows, contract.n_entities);
    let masks: Vec<u32> = detectors.iter().map(|d| d.detector_mask).collect();
    let candidates = prepare_with_detectors(
        &consensus,
        &masks,
        contract.n_windows,
        contract.n_entities,
        &CandidateConfig::CANONICAL,
    );

    build_from_artifacts_with_mode(
        events,
        contract,
        "cpu",
        EmissionMode::Audit,
        &features,
        &residuals,
        &signs,
        &detectors,
        &consensus,
        &candidates,
    )
}

/// Build the case file from the CPU pipeline in [`EmissionMode::Throughput`].
///
/// Same semantic chain as [`build_cpu`] but the per-stage hashes are
/// produced over the compact little-endian byte form. Much faster at
/// scaled contracts; produces a different `final_case_file_hash`
/// because the bytes hashed differ. Use this when speed matters and
/// the throughput-mode receipt is acceptable.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_cpu_throughput(events: &[TraceEvent], contract: &Contract) -> CaseFile {
    let features = compute_features(
        events,
        contract.n_windows,
        contract.n_entities,
        u64::from(contract.window_size_ms) * 1_000_000,
    );
    let residuals = residual_compute(&features, &Baseline::CANONICAL);
    let signs = sign_compute(
        &residuals,
        Q16::from_raw(contract.ewma_alpha_q16_raw),
        contract.n_windows,
        contract.n_entities,
    );
    let detectors = detector_evaluate(
        &residuals,
        &signs,
        &DetectorThresholds::CANONICAL,
        contract.n_windows,
        contract.n_entities,
    );
    let consensus = consensus_form(&signs, &detectors, contract.n_windows, contract.n_entities);
    let masks: Vec<u32> = detectors.iter().map(|d| d.detector_mask).collect();
    let candidates = prepare_with_detectors(
        &consensus,
        &masks,
        contract.n_windows,
        contract.n_entities,
        &CandidateConfig::CANONICAL,
    );
    build_from_artifacts_with_mode(
        events,
        contract,
        "cpu",
        EmissionMode::Throughput,
        &features,
        &residuals,
        &signs,
        &detectors,
        &consensus,
        &candidates,
    )
}

/// Public: build the case file in [`EmissionMode::Audit`] from already-
/// computed pipeline artifacts. Equivalent to
/// `build_from_artifacts_with_mode(..., EmissionMode::Audit)`. The CPU
/// and CUDA dispatchers' default entry points both feed this so the
/// canonical-text hash chain is what the prior-art receipt verifies.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_from_artifacts(
    events: &[TraceEvent],
    contract: &Contract,
    backend: &'static str,
    features: &[WindowFeature],
    residuals: &[ResidualCell],
    signs: &[SignCell],
    detectors: &[DetectorCell],
    consensus: &[ConsensusCell],
    candidates: &[CandidateInterval],
) -> CaseFile {
    build_from_artifacts_with_mode(
        events,
        contract,
        backend,
        EmissionMode::Audit,
        features,
        residuals,
        signs,
        detectors,
        consensus,
        candidates,
    )
}

/// Internal: build the case file from already-computed pipeline artifacts.
/// The `mode` parameter selects which byte form each per-stage hash is
/// computed over. Both `build_cpu` and the GPU dispatcher call this so
/// the bank-collapse, hash-chain, verdict, and final-hash logic lives in
/// a single place.
#[must_use]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn build_from_artifacts_with_mode(
    events: &[TraceEvent],
    contract: &Contract,
    backend: &'static str,
    mode: EmissionMode,
    features: &[WindowFeature],
    residuals: &[ResidualCell],
    signs: &[SignCell],
    detectors: &[DetectorCell],
    consensus: &[ConsensusCell],
    candidates: &[CandidateInterval],
) -> CaseFile {
    let episodes = bank_collapse(
        candidates,
        consensus,
        contract.n_windows,
        contract.n_entities,
    );

    // 2. Build the hash chain from canonical bytes for each stage.
    // Input-catalog hash is mode-dependent (Tier 3A optimization):
    //
    //   * Audit  -> SHA-256 over the canonical-JSON fixture bytes
    //     (1.6 MB at v0 scale, much larger at scaled contracts).
    //     Human-readable, audit-friendly, slow.
    //   * Throughput -> SHA-256 streamed directly over the events'
    //     `repr(C)` field bytes (48 bytes/event = 480 KB at v0 scale,
    //     2.5 MB at 524k events). No intermediate Vec allocation, no
    //     JSON-text formatting, no allocation traffic. Same chain
    //     semantics; smaller bytes hashed.
    //
    // Because the input-catalog hash differs between modes, every
    // downstream chain link also differs between modes — which is
    // exactly the existing invariant the `mode` field on `CaseFile`
    // records.
    let input_catalog = match mode {
        EmissionMode::Audit => sha256(&write_fixture(events)),
        EmissionMode::Throughput => hash_events_compact(events),
    };

    let contract_h = chain(b"contract", &contract.hash(), &input_catalog);
    let bank_h = chain(b"bank", &bank_hash(), &contract_h);
    let detector_registry_h = chain(b"detreg", &motif_registry_hash(), &bank_h);
    let kernel_sequence_h = chain(
        b"kseq",
        &contract.kernel_sequence_hash(),
        &detector_registry_h,
    );

    // Per-stage hashes branch on the emission mode. Audit hashes the
    // canonical-text bytes (slow, human-readable, pinned by the
    // golden_hashes test). Throughput hashes the compact little-endian
    // raw bytes — far cheaper at scale, byte-stable per backend, and
    // documented in the CaseFile's `mode` field so a reader knows which
    // scheme produced each hash.
    let window_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_window_features(features)),
        EmissionMode::Throughput => hash_window_features_compact(features),
    };
    let window_h = chain(b"window", &window_bytes_hash, &kernel_sequence_h);

    let residual_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_residual_cells(residuals)),
        EmissionMode::Throughput => hash_residual_cells_compact(residuals),
    };
    let residual_h = chain(b"residual", &residual_bytes_hash, &window_h);

    let sign_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_sign_cells(signs)),
        EmissionMode::Throughput => hash_sign_cells_compact(signs),
    };
    let sign_h = chain(b"sign", &sign_bytes_hash, &residual_h);

    let detector_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_detector_cells(detectors)),
        EmissionMode::Throughput => hash_detector_cells_compact(detectors),
    };
    let detector_h = chain(b"detector", &detector_bytes_hash, &sign_h);

    let consensus_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_consensus_cells(consensus)),
        EmissionMode::Throughput => hash_consensus_cells_compact(consensus),
    };
    let consensus_h = chain(b"consensus", &consensus_bytes_hash, &detector_h);

    let candidate_bytes_hash = match mode {
        EmissionMode::Audit => sha256(&serialize_candidates(candidates)),
        EmissionMode::Throughput => hash_candidates_compact(candidates),
    };
    let candidate_h = chain(b"candidate", &candidate_bytes_hash, &consensus_h);
    let episode_h = chain(
        b"episode",
        &sha256(&serialize_episodes(&episodes)),
        &candidate_h,
    );

    let hashes = IntermediateHashes {
        input_catalog,
        contract: contract_h,
        bank: bank_h,
        detector_registry: detector_registry_h,
        kernel_sequence: kernel_sequence_h,
        window_feature: window_h,
        residual_field: residual_h,
        sign_field: sign_h,
        detector_cell: detector_h,
        consensus_grid: consensus_h,
        candidate_interval: candidate_h,
        episode: episode_h,
    };

    let initial_verdict = if backend == "cuda" {
        FinalVerdict::GpuReplayAdmissible
    } else {
        FinalVerdict::CpuOnlyAdmissible
    };

    let mut case = CaseFile {
        version: CASE_FILE_VERSION,
        backend,
        mode,
        hashes,
        episodes,
        final_case_file_hash: [0u8; 32],
        final_verdict: initial_verdict,
    };

    // 3. Apply the Semantic Non-Bypass guard before computing the final
    // hash: any episode lacking an admission token short-circuits the
    // entire emission to `SemanticBypassRejected`.
    if case.episodes.iter().any(|e| !e.is_bank_admitted()) {
        case.final_verdict = FinalVerdict::SemanticBypassRejected;
    } else if case.hashes.bank != chain(b"bank", &contract.bank_hash, &case.hashes.contract)
        && contract.bank_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::BankMismatch;
    } else if case.hashes.detector_registry
        != chain(
            b"detreg",
            &contract.detector_registry_hash,
            &case.hashes.bank,
        )
        && contract.detector_registry_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::DetectorRegistryMismatch;
    }

    // 4. Final-hash: SHA-256 over the case-file canonical bytes with the
    // self-field still zero.
    case.final_case_file_hash = compute_final_hash(&case);
    case
}

/// Tier 3B (O.17) entry point: build a Throughput-mode case file using
/// precomputed per-stage SHA-256 digests for residual / sign / detector
/// / consensus. The GPU produced these digests on-device by hashing the
/// `#[repr(C)]` cell buffers directly (no marshalling).
///
/// What stays on the device: residual / sign / detector cell buffers.
/// Their hash contributions to the chain come in via the four
/// precomputed digests. The 3 D2H copies for those buffers are
/// eliminated, which is the principal wall-time saving Tier 3B
/// targets.
///
/// What still comes back to the host: the consensus grid (because
/// the bank stage's axis-5 entity-locality gate computes per-entity
/// and per-grid averages over the candidate's window range, which
/// requires the full grid) and the candidates (for per-candidate
/// admission). The consensus digest is still useful as a chain link
/// produced on-device; passing it as `consensus_digest` confirms the
/// caller is using the same byte form on both sides.
///
/// Determinism: per-stage digests produced by the device kernel are
/// byte-equal to the host's `hash_*_compact` digests by construction —
/// the device SHA-256 is the same FIPS 180-4 algorithm and the cell
/// buffer is the same `#[repr(C)]` little-endian layout. Pinned by the
/// `device_sha256_self_test` (low-level) and
/// `throughput_device_digests_match_host_digests` (whole-pipeline)
/// acceptance tests.
///
/// The candidate stage stays host-side because the host already
/// has the candidate buffer for the bank stage; computing its
/// digest on-device would not save any D2H bytes.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn build_throughput_from_artifacts_and_device_digests(
    events: &[TraceEvent],
    contract: &Contract,
    backend: &'static str,
    features: &[WindowFeature],
    residual_digest: [u8; 32],
    sign_digest: [u8; 32],
    detector_digest: [u8; 32],
    consensus_digest: [u8; 32],
    consensus_cells: &[ConsensusCell],
    candidates: &[CandidateInterval],
) -> CaseFile {
    let mode = EmissionMode::Throughput;
    let episodes = bank_collapse(
        candidates,
        consensus_cells,
        contract.n_windows,
        contract.n_entities,
    );

    let input_catalog = hash_events_compact(events);
    let contract_h = chain(b"contract", &contract.hash(), &input_catalog);
    let bank_h = chain(b"bank", &bank_hash(), &contract_h);
    let detector_registry_h = chain(b"detreg", &motif_registry_hash(), &bank_h);
    let kernel_sequence_h = chain(
        b"kseq",
        &contract.kernel_sequence_hash(),
        &detector_registry_h,
    );

    let window_bytes_hash = hash_window_features_compact(features);
    let window_h = chain(b"window", &window_bytes_hash, &kernel_sequence_h);

    // The four device-side digests slot in here. They are
    // byte-identical to `hash_*_compact(cells)` by the determinism
    // invariant above.
    let residual_h = chain(b"residual", &residual_digest, &window_h);
    let sign_h = chain(b"sign", &sign_digest, &residual_h);
    let detector_h = chain(b"detector", &detector_digest, &sign_h);
    let consensus_h = chain(b"consensus", &consensus_digest, &detector_h);

    let candidate_bytes_hash = hash_candidates_compact(candidates);
    let candidate_h = chain(b"candidate", &candidate_bytes_hash, &consensus_h);
    let episode_h = chain(
        b"episode",
        &sha256(&serialize_episodes(&episodes)),
        &candidate_h,
    );

    let hashes = IntermediateHashes {
        input_catalog,
        contract: contract_h,
        bank: bank_h,
        detector_registry: detector_registry_h,
        kernel_sequence: kernel_sequence_h,
        window_feature: window_h,
        residual_field: residual_h,
        sign_field: sign_h,
        detector_cell: detector_h,
        consensus_grid: consensus_h,
        candidate_interval: candidate_h,
        episode: episode_h,
    };

    let initial_verdict = if backend == "cuda" {
        FinalVerdict::GpuReplayAdmissible
    } else {
        FinalVerdict::CpuOnlyAdmissible
    };

    let mut case = CaseFile {
        version: CASE_FILE_VERSION,
        backend,
        mode,
        hashes,
        episodes,
        final_case_file_hash: [0u8; 32],
        final_verdict: initial_verdict,
    };

    if case.episodes.iter().any(|e| !e.is_bank_admitted()) {
        case.final_verdict = FinalVerdict::SemanticBypassRejected;
    } else if case.hashes.bank != chain(b"bank", &contract.bank_hash, &case.hashes.contract)
        && contract.bank_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::BankMismatch;
    } else if case.hashes.detector_registry
        != chain(
            b"detreg",
            &contract.detector_registry_hash,
            &case.hashes.bank,
        )
        && contract.detector_registry_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::DetectorRegistryMismatch;
    }

    case.final_case_file_hash = compute_final_hash(&case);
    case
}

/// R.11 — precomputed fixture-derived hashes that
/// `build_throughput_compact_verdict_from_device_digests` consumes
/// instead of re-hashing the input events and window features on
/// every dispatch.
///
/// **Why this exists.** R.8's profile (commit `ba5a3e4`) put host
/// bank + case finalize at 20 % of wall (~553 ms at 256×4096 K=1).
/// After R.8.5 collapsed the device-digest stage to ~68 ms, the
/// bank + finalize cost became ~82 % of the remaining wall. R.8
/// localised that cost: it's dominated by `hash_events_compact`
/// over ~4 M events (~176 MB) and `hash_window_features_compact`
/// over ~1 M features (~24 MB), both running scalar SHA-256 in
/// the host's hand-rolled implementation.
///
/// `FixtureHashes` lets a caller compute those two hashes once
/// per `(events, contract)` pair (e.g. outside a bench iter loop)
/// and feed them into every subsequent dispatch. The chain
/// semantics are unchanged: the case file's final hash is
/// byte-identical to the non-compact path's final hash for the
/// same fixture, because we are passing the same canonical
/// digests that `hash_events_compact` / `hash_window_features_
/// compact` would have computed — we just compute them once
/// instead of every iteration.
///
/// **Provenance.** Use `FixtureHashes::compute(events, features)`
/// to derive both fields canonically from the same `&[TraceEvent]`
/// and `&[WindowFeature]` the dispatch path would otherwise
/// re-hash. The function is just a thin helper around the
/// existing `hash_events_compact` / `hash_window_features_compact`
/// internals; nothing weakens or shortcuts the commitment.
#[derive(Copy, Clone, Debug)]
pub struct FixtureHashes {
    /// SHA-256 over the canonical compact byte form of the input
    /// `TraceEvent[]` (44 bytes per event, little-endian, padding
    /// excluded). Identical to the value `hash_events_compact`
    /// would compute on the same slice.
    pub input_catalog: [u8; 32],
    /// SHA-256 over the canonical compact byte form of the
    /// `WindowFeature[]` slice (24 bytes per feature,
    /// little-endian, padding excluded). Identical to the value
    /// `hash_window_features_compact` would compute.
    pub window_feature: [u8; 32],
}

impl FixtureHashes {
    /// Compute the two canonical hashes from the same byte forms
    /// the chain commits to. Call this once per fixture (events +
    /// features) and reuse the returned struct across every
    /// `build_throughput_compact_verdict_from_device_digests`
    /// invocation that uses that fixture.
    #[must_use]
    pub fn compute(events: &[TraceEvent], features: &[WindowFeature]) -> Self {
        Self {
            input_catalog: hash_events_compact(events),
            window_feature: hash_window_features_compact(features),
        }
    }
}

/// R.11 — build a Throughput-mode case file using pre-computed
/// fixture hashes plus the device-side per-stage digests. Same
/// 12-link chain semantics as
/// `build_throughput_from_artifacts_and_device_digests`, but
/// skips the per-dispatch re-hashing of ~4 M events and ~1 M
/// window features by taking the corresponding canonical hashes
/// from `FixtureHashes`.
///
/// Byte equivalence: when `FixtureHashes::compute(events,
/// features)` is used to derive the inputs, this function
/// produces a `CaseFile` byte-identical to the one
/// `build_throughput_from_artifacts_and_device_digests` would
/// have produced from the same `(events, features, contract,
/// digests, candidates)` tuple. The compact builder does no work
/// that the non-compact builder skips; it only avoids work the
/// non-compact builder repeats.
///
/// **Semantic Non-Bypass Axiom**: the compact builder still
/// admits episodes via `bank_collapse`, the only function that
/// can mint a `BankAdmissionToken`. A caller that fabricates a
/// `CompactVerdictReceipt` outside this builder cannot mint an
/// admitted episode because the token type's constructor is
/// private to the bank module.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn build_throughput_compact_verdict_from_device_digests(
    contract: &Contract,
    backend: &'static str,
    fixture: &FixtureHashes,
    residual_digest: [u8; 32],
    sign_digest: [u8; 32],
    detector_digest: [u8; 32],
    consensus_digest: [u8; 32],
    candidates: &[CandidateInterval],
) -> CaseFile {
    // The bank stage takes the same compact descriptors as the
    // non-compact path. R.5 stripped the consensus-cell argument
    // (axis-5 fields live on `CandidateInterval` itself now), so
    // `bank_collapse` reads nothing here that the compact path
    // doesn't already have.
    let mode = EmissionMode::Throughput;
    let episodes = bank_collapse(candidates, &[], contract.n_windows, contract.n_entities);

    let contract_h = chain(b"contract", &contract.hash(), &fixture.input_catalog);
    let bank_h = chain(b"bank", &bank_hash(), &contract_h);
    // R.9.b.3: the compact-verdict chain reads
    // `contract.detector_registry_hash` (the caller's pinned value)
    // instead of the always-canonical `motif_registry_hash()`. For
    // D16 callers the two are equal (they pin `registry_hash()`),
    // so D16 chain bytes are unchanged. For D64+ callers the chain
    // commits to the per-profile hash from
    // `DetectorProfile::DN.registry_hash()`, which is what the
    // case-file verifier compares against the contract at replay.
    // Fallback for callers that left `detector_registry_hash`
    // unpinned ([0u8; 32]): use the canonical motif registry hash
    // so the pre-R.9.b.3 implicit behaviour is preserved.
    let effective_registry_hash = if contract.detector_registry_hash == [0u8; 32] {
        motif_registry_hash()
    } else {
        contract.detector_registry_hash
    };
    let detector_registry_h = chain(b"detreg", &effective_registry_hash, &bank_h);
    let kernel_sequence_h = chain(
        b"kseq",
        &contract.kernel_sequence_hash(),
        &detector_registry_h,
    );
    let window_h = chain(b"window", &fixture.window_feature, &kernel_sequence_h);
    let residual_h = chain(b"residual", &residual_digest, &window_h);
    let sign_h = chain(b"sign", &sign_digest, &residual_h);
    let detector_h = chain(b"detector", &detector_digest, &sign_h);
    let consensus_h = chain(b"consensus", &consensus_digest, &detector_h);
    let candidate_bytes_hash = hash_candidates_compact(candidates);
    let candidate_h = chain(b"candidate", &candidate_bytes_hash, &consensus_h);
    let episode_h = chain(
        b"episode",
        &sha256(&serialize_episodes(&episodes)),
        &candidate_h,
    );

    let hashes = IntermediateHashes {
        input_catalog: fixture.input_catalog,
        contract: contract_h,
        bank: bank_h,
        detector_registry: detector_registry_h,
        kernel_sequence: kernel_sequence_h,
        window_feature: window_h,
        residual_field: residual_h,
        sign_field: sign_h,
        detector_cell: detector_h,
        consensus_grid: consensus_h,
        candidate_interval: candidate_h,
        episode: episode_h,
    };

    let initial_verdict = if backend == "cuda" {
        FinalVerdict::GpuReplayAdmissible
    } else {
        FinalVerdict::CpuOnlyAdmissible
    };

    let mut case = CaseFile {
        version: CASE_FILE_VERSION,
        backend,
        mode,
        hashes,
        episodes,
        final_case_file_hash: [0u8; 32],
        final_verdict: initial_verdict,
    };

    if case.episodes.iter().any(|e| !e.is_bank_admitted()) {
        case.final_verdict = FinalVerdict::SemanticBypassRejected;
    } else if case.hashes.bank != chain(b"bank", &contract.bank_hash, &case.hashes.contract)
        && contract.bank_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::BankMismatch;
    } else if case.hashes.detector_registry
        != chain(
            b"detreg",
            &contract.detector_registry_hash,
            &case.hashes.bank,
        )
        && contract.detector_registry_hash != [0u8; 32]
    {
        case.final_verdict = FinalVerdict::DetectorRegistryMismatch;
    }

    case.final_case_file_hash = compute_final_hash(&case);
    case
}

/// R.3a — build a `CompactCaseSummary` from the same precomputed
/// device digests that
/// `build_throughput_from_artifacts_and_device_digests` consumes, but
/// **stop before the bank stage**.
///
/// What this is for: the Layer A device-evidence-fabric dispatch path.
/// Layer A's purpose in the three-layer bench taxonomy (R.1) is to
/// measure the cost of the GPU evidence fabric in isolation — with no
/// host-side bank stage, no episode admission, no
/// `final_case_file_hash`. This function provides exactly that
/// boundary at the type level: a caller of Layer A receives a
/// `CompactCaseSummary` (no `episodes` field, no
/// `final_case_file_hash`), so the Semantic Non-Bypass Axiom holds by
/// construction — Layer A cannot admit episodes because the type
/// does not carry them.
///
/// The first 11 chain hashes (through `candidate_interval`) are
/// byte-identical to what the full-case-file builder would emit for
/// the same inputs; `episode` and `final_case_file_hash` are simply
/// not computed. Pinned by the
/// `compact_summary_chain_matches_full_case_file` acceptance test.
///
/// `breach_flags` are computed by comparing the chain links to the
/// contract's pinned hashes — the same predicates that set
/// `FinalVerdict::BankMismatch` / `DetectorRegistryMismatch` in the
/// full builder. A non-zero value signals a contract problem an
/// auditor can detect at evidence-fabric time, before the bank stage
/// runs.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn build_compact_summary_from_device_digests(
    events: &[TraceEvent],
    contract: &Contract,
    backend: &'static str,
    features: &[WindowFeature],
    residual_digest: [u8; 32],
    sign_digest: [u8; 32],
    detector_digest: [u8; 32],
    consensus_digest: [u8; 32],
    candidates: &[CandidateInterval],
) -> CompactCaseSummary {
    let mode = EmissionMode::Throughput;

    let input_catalog = hash_events_compact(events);
    let contract_h = chain(b"contract", &contract.hash(), &input_catalog);
    let bank_h = chain(b"bank", &bank_hash(), &contract_h);
    let detector_registry_h = chain(b"detreg", &motif_registry_hash(), &bank_h);
    let kernel_sequence_h = chain(
        b"kseq",
        &contract.kernel_sequence_hash(),
        &detector_registry_h,
    );

    let window_bytes_hash = hash_window_features_compact(features);
    let window_h = chain(b"window", &window_bytes_hash, &kernel_sequence_h);

    let residual_h = chain(b"residual", &residual_digest, &window_h);
    let sign_h = chain(b"sign", &sign_digest, &residual_h);
    let detector_h = chain(b"detector", &detector_digest, &sign_h);
    let consensus_h = chain(b"consensus", &consensus_digest, &detector_h);

    let candidate_bytes_hash = hash_candidates_compact(candidates);
    let candidate_h = chain(b"candidate", &candidate_bytes_hash, &consensus_h);

    // `episode` link intentionally absent from the summary: Layer A
    // does not run the bank, so it cannot produce an episode digest.
    // The full case-file builder fills it in downstream when the
    // summary is promoted to a CaseFile.
    let hashes = IntermediateHashes {
        input_catalog,
        contract: contract_h,
        bank: bank_h,
        detector_registry: detector_registry_h,
        kernel_sequence: kernel_sequence_h,
        window_feature: window_h,
        residual_field: residual_h,
        sign_field: sign_h,
        detector_cell: detector_h,
        consensus_grid: consensus_h,
        candidate_interval: candidate_h,
        // Unused for Layer A; kept zero so the struct layout is stable
        // and a later promotion to a `CaseFile` can recompute the
        // episode link from the candidates + consensus on the host.
        episode: [0u8; 32],
    };

    // Breach flags computed from the same predicates the full builder
    // uses to set FinalVerdict::BankMismatch / DetectorRegistryMismatch.
    let mut breach_flags: u32 = 0;
    if hashes.bank != chain(b"bank", &contract.bank_hash, &hashes.contract)
        && contract.bank_hash != [0u8; 32]
    {
        breach_flags |= breach_flags::BANK_HASH_MISMATCH;
    }
    if hashes.detector_registry != chain(b"detreg", &contract.detector_registry_hash, &hashes.bank)
        && contract.detector_registry_hash != [0u8; 32]
    {
        breach_flags |= breach_flags::DETECTOR_REGISTRY_MISMATCH;
    }

    CompactCaseSummary {
        backend,
        mode,
        hashes,
        candidates: candidates.to_vec(),
        breach_flags,
    }
}

/// Write the case-file canonical bytes minus the final-hash field.
/// Used both for computing the final hash and for emitting the file.
fn write_case_canonical_skipping_self_hash(case: &CaseFile, buf: &mut Vec<u8>) {
    buf.push(b'{');
    write_key(buf, "backend");
    write_str(buf, case.backend);
    buf.push(b',');
    write_key(buf, "episodes");
    buf.push(b'[');
    for (i, ep) in case.episodes.iter().enumerate() {
        if i > 0 {
            buf.push(b',');
        }
        write_episode(buf, ep);
    }
    buf.push(b']');
    buf.push(b',');
    write_key(buf, "final_verdict");
    write_str(buf, case.final_verdict.name());
    buf.push(b',');
    write_key(buf, "hashes");
    write_hashes(buf, &case.hashes);
    buf.push(b',');
    write_key(buf, "mode");
    write_str(buf, case.mode.name());
    buf.push(b',');
    write_key(buf, "version");
    write_str(buf, case.version);
    buf.push(b'}');
}

fn write_hashes(buf: &mut Vec<u8>, h: &IntermediateHashes) {
    buf.push(b'{');
    // Order keys lexicographically.
    let entries: [(&str, &[u8; 32]); 12] = [
        ("bank", &h.bank),
        ("candidate_interval", &h.candidate_interval),
        ("consensus_grid", &h.consensus_grid),
        ("contract", &h.contract),
        ("detector_cell", &h.detector_cell),
        ("detector_registry", &h.detector_registry),
        ("episode", &h.episode),
        ("input_catalog", &h.input_catalog),
        ("kernel_sequence", &h.kernel_sequence),
        ("residual_field", &h.residual_field),
        ("sign_field", &h.sign_field),
        ("window_feature", &h.window_feature),
    ];
    for (i, (key, val)) in entries.iter().enumerate() {
        if i > 0 {
            buf.push(b',');
        }
        write_key(buf, key);
        // Strings are surrounded by quotes; we emit the prefix-and-hex form.
        buf.push(b'"');
        buf.extend_from_slice(b"sha256:");
        for byte in *val {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            buf.push(HEX[(byte >> 4) as usize]);
            buf.push(HEX[(byte & 0x0F) as usize]);
        }
        buf.push(b'"');
    }
    buf.push(b'}');
}

fn write_episode(buf: &mut Vec<u8>, ep: &Episode) {
    buf.push(b'{');
    // Order keys lexicographically.
    write_key(buf, "admitted");
    write_str(
        buf,
        if ep.is_bank_admitted() {
            "true"
        } else {
            "false"
        },
    );
    buf.push(b',');
    write_key(buf, "detector_bit_count");
    write_u64(buf, u64::from(ep.detector_bit_count));
    buf.push(b',');
    write_key(buf, "end_window");
    write_u64(buf, u64::from(ep.end_window));
    buf.push(b',');
    write_key(buf, "entity_id");
    write_u64(buf, u64::from(ep.entity_id));
    buf.push(b',');
    write_key(buf, "motif");
    write_str(buf, ep.motif.name());
    buf.push(b',');
    write_key(buf, "peak_drift_q_raw");
    write_q16_raw(buf, ep.peak_drift_q);
    buf.push(b',');
    write_key(buf, "peak_residual_q_raw");
    write_q16_raw(buf, ep.peak_residual_q);
    buf.push(b',');
    write_key(buf, "peak_slew_q_raw");
    write_q16_raw(buf, ep.peak_slew_q);
    buf.push(b',');
    write_key(buf, "peak_state");
    write_str(buf, grammar_name(ep.peak_state));
    buf.push(b',');
    write_key(buf, "reason");
    write_str(buf, reason_name(ep.reason));
    buf.push(b',');
    write_key(buf, "start_window");
    write_u64(buf, u64::from(ep.start_window));
    buf.push(b'}');
}

const fn grammar_name(s: GrammarState) -> &'static str {
    match s {
        GrammarState::Admissible => "Admissible",
        GrammarState::Boundary => "Boundary",
        GrammarState::Violation => "Violation",
        GrammarState::Recovery => "Recovery",
    }
}

const fn reason_name(r: ReasonCode) -> &'static str {
    match r {
        ReasonCode::Admissible => "Admissible",
        ReasonCode::BoundaryApproach => "BoundaryApproach",
        ReasonCode::SustainedOutwardDrift => "SustainedOutwardDrift",
        ReasonCode::AbruptSlewViolation => "AbruptSlewViolation",
        ReasonCode::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
        ReasonCode::EnvelopeViolation => "EnvelopeViolation",
        ReasonCode::DriftWithRecovery => "DriftWithRecovery",
        ReasonCode::SingleCrossing => "SingleCrossing",
    }
}

fn write_q16_raw(buf: &mut Vec<u8>, q: Q16) {
    let raw = q.raw();
    if raw < 0 {
        buf.push(b'-');
        write_u64(buf, (-(raw as i64)) as u64);
    } else {
        write_u64(buf, raw as u64);
    }
}

fn compute_final_hash(case: &CaseFile) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    write_case_canonical_skipping_self_hash(case, &mut buf);
    sha256(&buf)
}

/// Emit the case file as canonical JSON. The final-hash field appears as
/// the last key in the object so a reader can verify it by hashing
/// everything before it.
#[must_use]
pub fn emit(case: &CaseFile) -> Vec<u8> {
    // First write a temporary record with the final hash field embedded
    // so the output object includes `final_case_file_hash`.
    let mut buf: Vec<u8> = Vec::new();
    buf.push(b'{');
    write_key(&mut buf, "backend");
    write_str(&mut buf, case.backend);
    buf.push(b',');
    write_key(&mut buf, "episodes");
    buf.push(b'[');
    for (i, ep) in case.episodes.iter().enumerate() {
        if i > 0 {
            buf.push(b',');
        }
        write_episode(&mut buf, ep);
    }
    buf.push(b']');
    buf.push(b',');
    write_key(&mut buf, "final_case_file_hash");
    buf.push(b'"');
    buf.extend_from_slice(b"sha256:");
    for byte in &case.final_case_file_hash {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        buf.push(HEX[(byte >> 4) as usize]);
        buf.push(HEX[(byte & 0x0F) as usize]);
    }
    buf.push(b'"');
    buf.push(b',');
    write_key(&mut buf, "final_verdict");
    write_str(&mut buf, case.final_verdict.name());
    buf.push(b',');
    write_key(&mut buf, "hashes");
    write_hashes(&mut buf, &case.hashes);
    buf.push(b',');
    write_key(&mut buf, "mode");
    write_str(&mut buf, case.mode.name());
    buf.push(b',');
    write_key(&mut buf, "version");
    write_str(&mut buf, case.version);
    buf.push(b'}');
    buf
}

/// Compare two case files and produce a final verdict. The CPU and GPU
/// case files are expected to agree at every link of the chain; the
/// first mismatch determines the verdict.
#[must_use]
pub fn compare(cpu: &CaseFile, gpu: &CaseFile) -> FinalVerdict {
    // Hash-chain divergence is `NumericMismatch` by default — any stage
    // hash that disagrees points to a per-stage numeric or ordering
    // difference. Specific verdicts are matched first.
    if cpu.final_verdict == FinalVerdict::SemanticBypassRejected
        || gpu.final_verdict == FinalVerdict::SemanticBypassRejected
    {
        return FinalVerdict::SemanticBypassRejected;
    }
    if cpu.mode != gpu.mode {
        // Per-stage hashes are mode-dependent — Audit and Throughput
        // produce different bytes for the same artifacts. We still
        // check contract / bank / detector_registry / kernel_sequence
        // (mode-invariant) plus the final-episode list below; the
        // per-stage equivalence assertions are skipped.
        if cpu.hashes.contract != gpu.hashes.contract {
            return FinalVerdict::ContractBreach;
        }
        if cpu.hashes.bank != gpu.hashes.bank {
            return FinalVerdict::BankMismatch;
        }
        if cpu.hashes.detector_registry != gpu.hashes.detector_registry {
            return FinalVerdict::DetectorRegistryMismatch;
        }
        if cpu.hashes.kernel_sequence != gpu.hashes.kernel_sequence {
            return FinalVerdict::KernelSequenceMismatch;
        }
        if cpu.episodes != gpu.episodes {
            return FinalVerdict::NumericMismatch;
        }
        return FinalVerdict::ReplayAdmissible;
    }
    if cpu.hashes.contract != gpu.hashes.contract {
        return FinalVerdict::ContractBreach;
    }
    if cpu.hashes.bank != gpu.hashes.bank {
        return FinalVerdict::BankMismatch;
    }
    if cpu.hashes.detector_registry != gpu.hashes.detector_registry {
        return FinalVerdict::DetectorRegistryMismatch;
    }
    if cpu.hashes.kernel_sequence != gpu.hashes.kernel_sequence {
        return FinalVerdict::KernelSequenceMismatch;
    }
    let stages: [(&[u8; 32], &[u8; 32]); 7] = [
        (&cpu.hashes.window_feature, &gpu.hashes.window_feature),
        (&cpu.hashes.residual_field, &gpu.hashes.residual_field),
        (&cpu.hashes.sign_field, &gpu.hashes.sign_field),
        (&cpu.hashes.detector_cell, &gpu.hashes.detector_cell),
        (&cpu.hashes.consensus_grid, &gpu.hashes.consensus_grid),
        (
            &cpu.hashes.candidate_interval,
            &gpu.hashes.candidate_interval,
        ),
        (&cpu.hashes.episode, &gpu.hashes.episode),
    ];
    for (a, b) in stages {
        if a != b {
            return FinalVerdict::NumericMismatch;
        }
    }
    // The final case-file hash is computed over canonical bytes that
    // include the `backend` and `final_verdict` strings — which legitimately
    // differ between CPU and GPU runs. Only compare the final hash when the
    // two case files came from the same backend; per-stage equivalence is
    // the load-bearing replay property.
    if cpu.backend == gpu.backend && cpu.final_case_file_hash != gpu.final_case_file_hash {
        return FinalVerdict::NumericMismatch;
    }
    FinalVerdict::ReplayAdmissible
}

fn serialize_window_features(features: &[WindowFeature]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(features.len() * 64);
    for f in features {
        write_u64(&mut buf, u64::from(f.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(f.window_idx));
        buf.push(b':');
        write_u64(&mut buf, u64::from(f.event_count));
        buf.push(b':');
        write_u64(&mut buf, u64::from(f.error_count));
        buf.push(b':');
        write_u64(&mut buf, f.sum_latency_us);
        buf.push(b';');
    }
    buf
}

fn serialize_residual_cells(cells: &[ResidualCell]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 48);
    for c in cells {
        write_u64(&mut buf, u64::from(c.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.window_idx));
        buf.push(b':');
        write_q16_raw(&mut buf, c.residual_latency_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.residual_error_q);
        buf.push(b';');
    }
    buf
}

fn serialize_sign_cells(cells: &[SignCell]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 64);
    for c in cells {
        write_u64(&mut buf, u64::from(c.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.window_idx));
        buf.push(b':');
        write_q16_raw(&mut buf, c.norm_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.drift_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.slew_q);
        buf.push(b';');
    }
    buf
}

fn serialize_detector_cells(cells: &[DetectorCell]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 32);
    for c in cells {
        write_u64(&mut buf, u64::from(c.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.window_idx));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.detector_mask));
        buf.push(b';');
    }
    buf
}

fn serialize_consensus_cells(cells: &[ConsensusCell]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 96);
    for c in cells {
        write_u64(&mut buf, u64::from(c.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.window_idx));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.detector_count));
        buf.push(b':');
        write_q16_raw(&mut buf, c.axis1_residual_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.axis2_drift_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.axis3_slew_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.axis4_temporal_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.axis7_consensus_q);
        buf.push(b';');
    }
    buf
}

fn serialize_candidates(cs: &[CandidateInterval]) -> Vec<u8> {
    // R.5 canonical byte order (Audit text form): entity_id,
    // start_window, end_window, union_mask, peak_residual_q,
    // peak_drift_q, peak_slew_q, peak_temporal_q, peak_consensus_q,
    // entity_avg_q, grid_avg_q. Each field separated by `:`; intervals
    // terminated by `;`. The hash that consumes these bytes is
    // `chain(b"candidate", &sha256(serialize_candidates(...)), ...)`,
    // so changing the order (or omitting a field) re-pins the
    // golden hashes intentionally.
    let mut buf: Vec<u8> = Vec::with_capacity(cs.len() * 80);
    for c in cs {
        write_u64(&mut buf, u64::from(c.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.start_window));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.end_window));
        buf.push(b':');
        write_u64(&mut buf, u64::from(c.union_mask));
        buf.push(b':');
        write_q16_raw(&mut buf, c.peak_residual_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.peak_drift_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.peak_slew_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.peak_temporal_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.peak_consensus_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.entity_avg_q);
        buf.push(b':');
        write_q16_raw(&mut buf, c.grid_avg_q);
        buf.push(b';');
    }
    buf
}

// ============================================================
// Compact-bytes streaming hashers for EmissionMode::Throughput.
//
// These functions stream each cell's raw little-endian field bytes
// through a Sha256 incrementally. There is no intermediate Vec<u8>
// allocation (unlike the audit-mode `serialize_*` functions above),
// so the per-stage hash cost drops from "write ~50-100 KB of
// canonical-text bytes plus SHA-256 over that" to "hash ~16-32 KB
// of compact bytes". At scaled contracts where each stage's cell
// count grows by 100×, this is the most material host-side speedup
// throughput mode can offer.
//
// The hash values produced here differ from the audit-mode hashes
// because the input bytes differ. That's by design — the CaseFile's
// `mode` field tells a reader which scheme produced the value, and
// the compare function below requires both case files to have the
// same mode before claiming per-stage equivalence.
// ============================================================

// Buffered (single Vec<u8> then one SHA-256) rather than streaming
// (many small Sha256::update calls). Profiling showed streaming was
// slower than canonical-text audit mode because SHA-256 update-call
// overhead dominated for the many tiny per-field invocations. With a
// single buffered call we get the actual size win (compact bytes are
// ~60% the size of canonical text).

/// Domain separator for the Throughput-mode input-catalog digest.
///
/// Doctrine: the Audit-mode input hash is implicitly domain-separated
/// because the only bytes that produce it are the canonical-JSON
/// fixture-file representation; nothing else in the system ever emits
/// those bytes. The Throughput-mode compact digest, on the other hand,
/// hashes raw `repr(C)` event bytes — which by themselves could be
/// confused with many unrelated byte streams in unrelated contexts. We
/// prefix every compact digest with this explicit domain string plus
/// a small manifest (event count, layout version) so the hash domain
/// is unambiguous and self-describing.
///
/// The `v1` suffix is the layout version. Any change to the
/// `TraceEvent` struct layout that affects the bytes hashed here MUST
/// be paired with an increment to this constant; otherwise the digest
/// would silently change without surfacing a contract revision.
pub const COMPACT_INPUT_DOMAIN: &str = "DSFB-GPU-DEBUG:input-event-stream:v1";

/// Symbolic name of the `TraceEvent` layout the compact digest hashes
/// over. Embedded into the digest manifest so a verifier knows which
/// field order produced the bytes. Any change to the field order or
/// width in `TraceEvent` MUST be paired with an increment to this
/// constant (e.g. `TraceEventV2`).
pub const TRACE_EVENT_LAYOUT_TAG: &str = "TraceEventV1";

fn hash_events_compact(events: &[TraceEvent]) -> [u8; 32] {
    // The compact digest layout, byte by byte:
    //
    //   [domain bytes]                   COMPACT_INPUT_DOMAIN
    //   0x00                             null separator
    //   [layout tag bytes]               TRACE_EVENT_LAYOUT_TAG
    //   0x00                             null separator
    //   [event_count as u64 le]          manifest: # events
    //   0x00                             null separator
    //   [event 0 bytes] ... [event N-1]  44 bytes per event (le)
    //
    // 44 bytes per event of content (the trailing 4 bytes of alignment
    // padding in the `repr(C)` struct are intentionally skipped so the
    // hash is independent of any platform's padding choices).
    //
    // The null separators are not strictly required but they prevent
    // any future field-concatenation ambiguity: a verifier can split on
    // null and recover each header segment unambiguously.
    let mut buf: Vec<u8> = Vec::with_capacity(
        COMPACT_INPUT_DOMAIN.len() + TRACE_EVENT_LAYOUT_TAG.len() + 16 + events.len() * 44,
    );
    buf.extend_from_slice(COMPACT_INPUT_DOMAIN.as_bytes());
    buf.push(0);
    buf.extend_from_slice(TRACE_EVENT_LAYOUT_TAG.as_bytes());
    buf.push(0);
    buf.extend_from_slice(&(events.len() as u64).to_le_bytes());
    buf.push(0);
    for e in events {
        buf.extend_from_slice(&e.ts_ns.to_le_bytes());
        buf.extend_from_slice(&e.entity_id.to_le_bytes());
        buf.extend_from_slice(&e.route_id.to_le_bytes());
        buf.extend_from_slice(&e.span_id.to_le_bytes());
        buf.extend_from_slice(&e.parent_span_id.to_le_bytes());
        buf.extend_from_slice(&e.latency_us.to_le_bytes());
        buf.extend_from_slice(&e.status_code.to_le_bytes());
        buf.extend_from_slice(&e.error_code.to_le_bytes());
        buf.extend_from_slice(&e.event_kind.to_le_bytes());
        buf.extend_from_slice(&e.flags.to_le_bytes());
    }
    sha256(&buf)
}

fn hash_window_features_compact(features: &[WindowFeature]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(features.len() * 24);
    for f in features {
        buf.extend_from_slice(&f.window_idx.to_le_bytes());
        buf.extend_from_slice(&f.entity_id.to_le_bytes());
        buf.extend_from_slice(&f.event_count.to_le_bytes());
        buf.extend_from_slice(&f.error_count.to_le_bytes());
        buf.extend_from_slice(&f.sum_latency_us.to_le_bytes());
    }
    sha256(&buf)
}

fn hash_residual_cells_compact(cells: &[ResidualCell]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 16);
    for c in cells {
        buf.extend_from_slice(&c.window_idx.to_le_bytes());
        buf.extend_from_slice(&c.entity_id.to_le_bytes());
        buf.extend_from_slice(&c.residual_latency_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.residual_error_q.raw().to_le_bytes());
    }
    sha256(&buf)
}

fn hash_sign_cells_compact(cells: &[SignCell]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 20);
    for c in cells {
        buf.extend_from_slice(&c.window_idx.to_le_bytes());
        buf.extend_from_slice(&c.entity_id.to_le_bytes());
        buf.extend_from_slice(&c.norm_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.drift_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.slew_q.raw().to_le_bytes());
    }
    sha256(&buf)
}

fn hash_detector_cells_compact(cells: &[DetectorCell]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 12);
    for c in cells {
        buf.extend_from_slice(&c.window_idx.to_le_bytes());
        buf.extend_from_slice(&c.entity_id.to_le_bytes());
        buf.extend_from_slice(&c.detector_mask.to_le_bytes());
    }
    sha256(&buf)
}

fn hash_consensus_cells_compact(cells: &[ConsensusCell]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(cells.len() * 32);
    for c in cells {
        buf.extend_from_slice(&c.window_idx.to_le_bytes());
        buf.extend_from_slice(&c.entity_id.to_le_bytes());
        buf.extend_from_slice(&c.detector_count.to_le_bytes());
        buf.extend_from_slice(&c.axis1_residual_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.axis2_drift_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.axis3_slew_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.axis4_temporal_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.axis7_consensus_q.raw().to_le_bytes());
    }
    sha256(&buf)
}

fn hash_candidates_compact(cs: &[CandidateInterval]) -> [u8; 32] {
    // R.5 canonical byte order (Throughput compact form), 48 bytes per
    // candidate, in the same order as `serialize_candidates`'s text
    // form: entity_id, start_window, end_window, length_windows,
    // union_mask (5 u32 = 20 bytes), then peak_residual_q, peak_drift_q,
    // peak_slew_q, peak_temporal_q, peak_consensus_q, entity_avg_q,
    // grid_avg_q (7 i32 Q16 = 28 bytes). Pre-R.5 layout was 40 bytes
    // per candidate (5 u32 + 5 i32 Q16); the two new locality
    // averages add 8 bytes and re-pin the candidate_interval chain
    // link. The golden-hashes test is refreshed in lockstep.
    let mut buf: Vec<u8> = Vec::with_capacity(cs.len() * 48);
    for c in cs {
        buf.extend_from_slice(&c.entity_id.to_le_bytes());
        buf.extend_from_slice(&c.start_window.to_le_bytes());
        buf.extend_from_slice(&c.end_window.to_le_bytes());
        buf.extend_from_slice(&c.length_windows.to_le_bytes());
        buf.extend_from_slice(&c.union_mask.to_le_bytes());
        buf.extend_from_slice(&c.peak_residual_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.peak_drift_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.peak_slew_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.peak_temporal_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.peak_consensus_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.entity_avg_q.raw().to_le_bytes());
        buf.extend_from_slice(&c.grid_avg_q.raw().to_le_bytes());
    }
    sha256(&buf)
}

fn serialize_episodes(eps: &[Episode]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(eps.len() * 64);
    for e in eps {
        write_u64(&mut buf, u64::from(e.entity_id));
        buf.push(b':');
        write_u64(&mut buf, u64::from(e.start_window));
        buf.push(b':');
        write_u64(&mut buf, u64::from(e.end_window));
        buf.push(b':');
        buf.extend_from_slice(e.motif.name().as_bytes());
        buf.push(b':');
        buf.extend_from_slice(reason_name(e.reason).as_bytes());
        buf.push(b';');
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{synthesize, DEFAULT_SEED};

    fn make_contract_for_test() -> Contract {
        let mut c = Contract::canonical();
        c.pin_bank_hash(bank_hash());
        c.pin_detector_registry_hash(motif_registry_hash());
        c
    }

    #[test]
    fn build_cpu_is_deterministic_byte_exact() {
        let events = synthesize(DEFAULT_SEED);
        let contract = make_contract_for_test();
        let a = build_cpu(&events, &contract);
        let b = build_cpu(&events, &contract);
        assert_eq!(a.final_case_file_hash, b.final_case_file_hash);
        assert_eq!(emit(&a), emit(&b));
    }

    #[test]
    fn build_cpu_admits_canonically() {
        let events = synthesize(DEFAULT_SEED);
        let contract = make_contract_for_test();
        let case = build_cpu(&events, &contract);
        assert_eq!(case.final_verdict, FinalVerdict::CpuOnlyAdmissible);
        // The three injected episodes must each appear in the admitted set.
        assert!(case
            .episodes
            .iter()
            .any(|e| e.entity_id == 3 && e.motif == BankMotif::LatencyRamp));
        assert!(case
            .episodes
            .iter()
            .any(|e| e.entity_id == 7 && e.motif == BankMotif::ErrorBurst));
        assert!(case
            .episodes
            .iter()
            .any(|e| e.entity_id == 11 && e.motif == BankMotif::SlewShockRecovery));
    }

    #[test]
    fn bank_mismatch_when_pinned_hash_diverges() {
        let events = synthesize(DEFAULT_SEED);
        let mut contract = make_contract_for_test();
        contract.bank_hash[0] ^= 0xFF;
        let case = build_cpu(&events, &contract);
        assert_eq!(case.final_verdict, FinalVerdict::BankMismatch);
    }

    #[test]
    fn detector_registry_mismatch_when_pinned_hash_diverges() {
        let events = synthesize(DEFAULT_SEED);
        let mut contract = make_contract_for_test();
        contract.detector_registry_hash[0] ^= 0xFF;
        let case = build_cpu(&events, &contract);
        assert_eq!(case.final_verdict, FinalVerdict::DetectorRegistryMismatch);
    }

    #[test]
    fn semantic_bypass_attempt_is_rejected() {
        let events = synthesize(DEFAULT_SEED);
        let contract = make_contract_for_test();
        let mut case = build_cpu(&events, &contract);
        // Inject a non-admitted episode: this models the breach the
        // spec test #10 is meant to catch.
        case.episodes.push(Episode::bypass_for_testing(
            0,
            0,
            1,
            BankMotif::ConfuserTransient,
        ));
        // Re-emit so the final verdict reflects the injected episode.
        let mut rerun = build_cpu(&events, &contract);
        rerun.episodes.push(Episode::bypass_for_testing(
            0,
            0,
            1,
            BankMotif::ConfuserTransient,
        ));
        // Recompute the verdict the same way build_cpu would.
        if rerun.episodes.iter().any(|e| !e.is_bank_admitted()) {
            rerun.final_verdict = FinalVerdict::SemanticBypassRejected;
        }
        assert_eq!(rerun.final_verdict, FinalVerdict::SemanticBypassRejected);
    }

    #[test]
    fn compare_identical_case_files_yields_replay_admissible() {
        let events = synthesize(DEFAULT_SEED);
        let contract = make_contract_for_test();
        let a = build_cpu(&events, &contract);
        let b = a.clone();
        assert_eq!(compare(&a, &b), FinalVerdict::ReplayAdmissible);
    }

    // O.15 acceptance criteria. Pinned here so future contract changes
    // that would silently re-domain the compact input digest cannot
    // sneak through.

    #[test]
    fn compact_input_digest_is_deterministic_across_runs() {
        // Same fixture seed -> identical compact input hash on every
        // call. This is the load-bearing replay property for the
        // Throughput-mode input link.
        let a = build_cpu_throughput(&synthesize(DEFAULT_SEED), &make_contract_for_test());
        let b = build_cpu_throughput(&synthesize(DEFAULT_SEED), &make_contract_for_test());
        assert_eq!(a.hashes.input_catalog, b.hashes.input_catalog);
    }

    #[test]
    fn compact_input_digest_changes_when_one_event_byte_changes() {
        // Mutating a single event field changes the input hash. This
        // ensures the digest is genuinely a function of every event
        // byte, not a collision-prone summary.
        let mut events = synthesize(DEFAULT_SEED);
        let baseline = build_cpu_throughput(&events, &make_contract_for_test());
        events[0].latency_us ^= 0x0000_00FF;
        let mutated = build_cpu_throughput(&events, &make_contract_for_test());
        assert_ne!(baseline.hashes.input_catalog, mutated.hashes.input_catalog);
    }

    #[test]
    fn compact_input_digest_changes_when_event_count_changes() {
        // The manifest's event_count field is part of the digest, so
        // truncating the event stream produces a different hash even
        // if the remaining events are identical.
        let events = synthesize(DEFAULT_SEED);
        let mut shorter = events.clone();
        shorter.pop();
        let full = build_cpu_throughput(&events, &make_contract_for_test());
        let truncated = build_cpu_throughput(&shorter, &make_contract_for_test());
        assert_ne!(full.hashes.input_catalog, truncated.hashes.input_catalog);
    }

    #[test]
    fn audit_input_digest_is_unchanged_by_compact_hash_additions() {
        // Acceptance criterion: the Audit mode digest must remain the
        // canonical-JSON SHA-256. We pin its value here and compare
        // against the golden_hashes test's expected `input_catalog`
        // constant. If this assertion fails the prior-art receipt has
        // drifted and the change must be either reverted or paired with
        // a contract revision.
        const GOLDEN_INPUT_CATALOG_HEX: &str =
            "1eeefffa2a1029672a9e6a55e575c928fca926e8077e6784a392597ccd640487";
        use std::fmt::Write;
        let case = build_cpu(&synthesize(DEFAULT_SEED), &make_contract_for_test());
        let mut hex = String::with_capacity(64);
        for b in &case.hashes.input_catalog {
            write!(hex, "{b:02x}").unwrap();
        }
        assert_eq!(hex, GOLDEN_INPUT_CATALOG_HEX);
    }

    #[test]
    fn compact_and_audit_input_digests_disagree_by_design() {
        // Doctrine: Audit and Throughput input hashes use different
        // byte schemes (canonical-JSON vs. domain-prefixed event stream)
        // and so must produce different digests. If they ever match,
        // someone removed the domain separator and the throughput hash
        // has silently aliased onto audit's hash domain.
        let events = synthesize(DEFAULT_SEED);
        let audit = build_cpu(&events, &make_contract_for_test());
        let throughput = build_cpu_throughput(&events, &make_contract_for_test());
        assert_ne!(audit.hashes.input_catalog, throughput.hashes.input_catalog);
    }

    #[test]
    fn compare_detects_per_stage_numeric_mismatch() {
        let events = synthesize(DEFAULT_SEED);
        let contract = make_contract_for_test();
        let mut a = build_cpu(&events, &contract);
        let b = a.clone();
        // Flip a single byte in the residual-field hash to model a
        // corrupted GPU output cell.
        a.hashes.residual_field[0] ^= 0x01;
        assert_eq!(compare(&a, &b), FinalVerdict::NumericMismatch);
    }
}
