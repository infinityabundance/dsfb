//! T.12.e — Signal Processing / Spectral / Wavelet: the fifth
//! real literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.e files the Signal Processing / Spectral / Wavelet
//! > amendment proposal. It admits only deterministic transform-
//! > based primitives whose sampling, windowing, normalization,
//! > band, boundary, and template laws are declared; resolves
//! > collisions with existing SEED records; classifies transform
//! > variants as parameterizations; rejects randomized or
//! > learned spectral claims unless deterministically reduced;
//! > and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"In spectral
//! detectors, the transform law is the detector. No transform
//! law, no canonical admission."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.e's design began with a grep of [`crate::seed::SEED`]
//! for every spectral candidate. The walk found **five**
//! signal / spectral primitives already canonical:
//!
//! * **FFT band-energy anomaly** at SEED id 12 — the most
//!   fundamental power-spectrum-based primitive. The shared
//!   spectral-transform ancestor recognised by the
//!   `SignalProcessing` source class.
//! * **Residual envelope exit** at SEED id 22 — envelope
//!   detection by residual-magnitude exit. Catches
//!   "envelope-detector" aliases.
//! * **Spectral entropy** at SEED id 38.
//! * **Wavelet coefficient energy** at SEED id 39.
//! * **Autocorrelation-coefficient break** at SEED id 40.
//!
//! All five become `ExistingCanonicalAuthorityResolution`
//! records under the `SignalProcessing` source class, each with
//! its specific transform-law contract declared in the reason
//! text. Of the remaining panel candidates the court ruled:
//!
//! * **Spectral centroid shift** is structurally distinct: a
//!   first-moment-of-power-spectrum decision functional with
//!   declared power-spectrum convention, frequency-bin mapping,
//!   first-moment formula, and sampling law. `CanonicalAddition`
//!   at reserved id 5501.
//! * **Wavelet packet energy** is structurally distinct from
//!   wavelet coefficient energy (SEED 39): the decomposition
//!   functional is a full packet-tree expansion rather than a
//!   single-level coefficient set. Declared wavelet-family,
//!   packet-tree-depth, energy convention, boundary handling,
//!   and sampling-law contract. `CanonicalAddition` at reserved
//!   id 5502.
//! * **STFT ridge shift** is structurally distinct: tracking
//!   the location of spectral peaks over time. Declared window
//!   function, window length, hop / overlap law, ridge
//!   selection law, extrapolation handling, and sampling law
//!   contract. `CanonicalAddition` at reserved id 5503.
//! * **Cepstral anomaly** is structurally distinct: inverse
//!   FFT of log power spectrum. Declared FFT convention + log
//!   base + real-cepstrum vs complex-cepstrum + sampling rate.
//!   `CanonicalAddition` at reserved id 5504.
//! * **Matched filter residual** is structurally distinct:
//!   cross-correlation with a declared template. Declared
//!   template provenance + sampling-rate match + normalization
//!   convention. `CanonicalAddition` at reserved id 5505.
//! * **Hilbert amplitude anomaly** is structurally distinct:
//!   analytic-signal extraction (FFT-based or filter-based)
//!   and amplitude envelope. Different decision functional
//!   from the residual envelope exit (which is direct-residual
//!   based). Declared analytic-signal-extraction method +
//!   sampling law. `CanonicalAddition` at reserved id 5506.
//! * **FFT bandpower variant** is `ParameterizationOf(FFT band-
//!   energy anomaly, SEED 12)` — band-edge / window-function /
//!   normalization parameterization of FFT band energy. Reserved
//!   id 5507.
//! * **Wavelet family variant** is `ParameterizationOf(wavelet
//!   coefficient energy, SEED 39)` — declared specific wavelet
//!   family (Daubechies-N, Symlets, Coiflets, Haar) + level.
//!   Reserved id 5508.
//! * **STFT window/hop variant** is `ParameterizationOf(STFT
//!   ridge shift, 5503)` — declared specific window function
//!   and hop fraction. Reserved id 5509.
//! * **Randomized spectral projection** (random Fourier features
//!   / random spectral subspace approximations) is randomized
//!   in origin. Acknowledged in `proposed_primitives` at
//!   reserved id 5510 but `RejectedNotDeterministic` — admitted
//!   neither to SEED nor to `new_canonical_records` unless a
//!   future T.12.x proposal admits a
//!   `Deterministic_Spectral_Projection_Proxy` with the seed,
//!   projection matrix definition, dimension, and numeric mode
//!   declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! T.12.e exercises all five panel-locked court-delta categories
//! (the wire-name set closed at T.12.d remains closed):
//!
//! * `CanonicalAddition` ×6 — Spectral centroid shift, Wavelet
//!   packet energy, STFT ridge shift, Cepstral anomaly, Matched
//!   filter residual, Hilbert amplitude anomaly.
//! * `ExistingCanonicalAuthorityResolution` ×5 — FFT band-
//!   energy (12), Residual envelope exit (22), Spectral entropy
//!   (38), Wavelet coefficient energy (39), Autocorrelation
//!   break (40).
//! * `DomainTransferOf` ×1 — FFT band-energy (12) as the shared
//!   spectral-transform ancestor for the `SignalProcessing`
//!   source class.
//! * `ParameterizationOf` ×3 — FFT bandpower variant
//!   (ParameterizationOf SEED 12), wavelet family variant
//!   (ParameterizationOf SEED 39), STFT window/hop variant
//!   (ParameterizationOf 5503).
//! * `RejectedNotDeterministic` ×1 — randomized spectral
//!   projection (5510).
//!
//! Total: 6 + 5 + 1 + 3 + 1 = **16 dedup-court records**.
//!
//! ## Transform-law contract discipline
//!
//! Every CanonicalAddition + ExistingCanonicalAuthorityResolution
//! record's reason text declares its specific transform-law
//! contract — without which the detector is meaningless.
//! Declared contract fields by primitive:
//!
//! * **FFT band-energy** (SEED 12) — window function +
//!   normalization + band definition + sampling law.
//! * **Spectral entropy** (SEED 38) — bin definition +
//!   power-mass normalization + log base + sampling law.
//! * **Wavelet coefficient energy** (SEED 39) — wavelet
//!   family + level + boundary handling + energy convention.
//! * **Autocorrelation break** (SEED 40) — lag set +
//!   normalization + window.
//! * **Residual envelope exit** (SEED 22) — envelope-
//!   extraction method + threshold.
//! * **Spectral centroid shift** (5501) — power-spectrum
//!   convention + frequency-bin mapping + first-moment formula.
//! * **Wavelet packet energy** (5502) — wavelet family +
//!   packet-tree depth + energy convention + boundary handling.
//! * **STFT ridge shift** (5503) — window + window length +
//!   hop / overlap + ridge selection law + extrapolation.
//! * **Cepstral anomaly** (5504) — FFT convention + log base
//!   + real / complex cepstrum.
//! * **Matched filter residual** (5505) — template provenance
//!   + sampling-rate match + normalization.
//! * **Hilbert amplitude anomaly** (5506) — analytic-signal
//!   extraction (FFT / filter) + sampling law.
//! * **Randomized spectral projection** (5510 rejection) —
//!   the four reduction requirements (seed + projection matrix
//!   definition + dimension + numeric mode) must be declared
//!   before any future admission.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11/S1.3/T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.e spectral
//!   `corpus_amendment_proposal_hash_v1` distinct from every
//!   prior T.12.x proposal hash.
//!
//! ## Discipline
//!
//! Same `no-silent-court-logic` doctrine as every prior T.12.x;
//! every `pub` item AND every private helper carries a doc
//! comment whose first sentence states the WHY for a future
//! engineer; 10-step ritual; no `--no-verify`.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    CorpusAmendmentProposal, ProposalStatus, ProposedAliasClaim, ProposedDedupRecord,
    ProposedGenealogyEdge, ProposedPrimitive, ProposedSourceRef, ProposerRole, RejectionRecord,
    SourceClass,
};
use crate::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Reserved id constants (panel-locked)
// ---------------------------------------------------------------

/// Reserved canonical id for Spectral centroid shift. 5501..5510
/// is the T.12.e bucket (T.12.a 5001+, T.12.b 5201+, T.12.c
/// 5301+, T.12.d 5401+).
pub const SPECTRAL_CENTROID_RESERVED_CANONICAL_ID: u32 = 5501;

/// Reserved canonical id for Wavelet packet energy.
pub const WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID: u32 = 5502;

/// Reserved canonical id for STFT ridge shift.
pub const STFT_RIDGE_RESERVED_CANONICAL_ID: u32 = 5503;

/// Reserved canonical id for Cepstral anomaly.
pub const CEPSTRAL_RESERVED_CANONICAL_ID: u32 = 5504;

/// Reserved canonical id for Matched filter residual.
pub const MATCHED_FILTER_RESERVED_CANONICAL_ID: u32 = 5505;

/// Reserved canonical id for Hilbert amplitude anomaly.
pub const HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID: u32 = 5506;

/// Reserved id for FFT bandpower variant.
/// `ParameterizationOf(FFT band-energy, SEED 12)`.
pub const FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID: u32 = 5507;

/// Reserved id for wavelet family variant.
/// `ParameterizationOf(wavelet coefficient energy, SEED 39)`.
pub const WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID: u32 = 5508;

/// Reserved id for STFT window/hop variant.
/// `ParameterizationOf(STFT ridge shift, 5503)`.
pub const STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID: u32 = 5509;

/// Reserved id for Randomized spectral projection
/// (`RejectedNotDeterministic`). Random Fourier features /
/// random spectral subspace approximations are randomized in
/// origin. A future T.12.x may admit a
/// `Deterministic_Spectral_Projection_Proxy` canonical only
/// with seed + projection matrix definition + dimension +
/// numeric mode declared.
pub const RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID: u32 = 5510;

// Existing SEED canonical ids referenced by the T.12.e
// cross-class authority resolutions.

/// FFT band-energy anomaly — SEED canonical id 12. The most
/// fundamental power-spectrum-based primitive; shared spectral-
/// transform ancestor.
pub const FFT_BAND_ENERGY_SEED_ID: u32 = 12;

/// Residual envelope exit — SEED canonical id 22.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// Spectral entropy — SEED canonical id 38.
pub const SPECTRAL_ENTROPY_SEED_ID: u32 = 38;

/// Wavelet coefficient energy — SEED canonical id 39.
pub const WAVELET_COEFFICIENT_ENERGY_SEED_ID: u32 = 39;

/// Autocorrelation-coefficient break — SEED canonical id 40.
pub const AUTOCORRELATION_BREAK_SEED_ID: u32 = 40;

// ---------------------------------------------------------------
// Panel-locked court-delta category wire names
// ---------------------------------------------------------------

/// `CanonicalAddition`.
pub const CATEGORY_CANONICAL_ADDITION: &str = "CanonicalAddition";

/// `ExistingCanonicalAuthorityResolution`.
pub const CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION: &str =
    "ExistingCanonicalAuthorityResolution";

/// `DomainTransferOf`.
pub const CATEGORY_DOMAIN_TRANSFER_OF: &str = "DomainTransferOf";

/// `ParameterizationOf`.
pub const CATEGORY_PARAMETERIZATION_OF: &str = "ParameterizationOf";

/// `RejectedNotDeterministic`.
pub const CATEGORY_REJECTED_NOT_DETERMINISTIC: &str = "RejectedNotDeterministic";

// ---------------------------------------------------------------
// Builders for the spectral expansion batch
// ---------------------------------------------------------------

/// Build the spectral `CorpusExpansionBatch` body: 10 proposed
/// primitives (6 canonical + 3 parameterization + 1 rejection),
/// 0 alias claims, 16 dedup-court records, 9 genealogy edges,
/// 8 source refs.
fn build_spectral_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_e_spectral_first_proposal",
        SourceClass::SignalProcessing,
        spectral_proposed_primitives(),
        spectral_proposed_aliases(),
        spectral_proposed_dedup_records(),
        spectral_proposed_genealogy_edges(),
        spectral_proposed_source_refs(),
    )
}

/// Ten proposed primitives: six genuinely new canonicals + three
/// parameterization shells + one rejection shell.
fn spectral_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SPECTRAL_CENTROID_RESERVED_CANONICAL_ID),
            display_name: "Spectral centroid shift",
            motivation: "First-moment-of-power-spectrum decision functional. \
                 Structurally distinct from spectral entropy (information-theoretic \
                 statistic) and from FFT band-energy (banded sum). Declared \
                 transform-law contract: power-spectrum convention + frequency-bin \
                 mapping + first-moment formula + sampling law. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID),
            display_name: "Wavelet packet energy",
            motivation: "Full packet-tree expansion (vs single-level wavelet \
                 coefficient energy at SEED id 39). The decomposition functional \
                 is structurally distinct: a packet tree decomposes BOTH detail \
                 and approximation coefficients recursively, exposing higher-\
                 frequency-resolution bands the single-level decomposition does \
                 not. Declared transform-law contract: wavelet family (Daubechies / \
                 Symlets / Coiflets) + packet-tree depth + energy convention + \
                 boundary handling + sampling law. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(STFT_RIDGE_RESERVED_CANONICAL_ID),
            display_name: "STFT ridge shift",
            motivation: "Short-time Fourier transform ridge tracking: location of \
                 spectral peak as a function of time. Structurally distinct from \
                 FFT band-energy (per-frame frequency-bin energy) and from spectral \
                 centroid (first-moment over frequency). Declared transform-law \
                 contract: window function + window length + hop / overlap law + \
                 ridge selection law (max-magnitude / max-energy / band-restricted) \
                 + extrapolation handling for missing ridges + sampling law. \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CEPSTRAL_RESERVED_CANONICAL_ID),
            display_name: "Cepstral anomaly",
            motivation: "Inverse FFT of log power spectrum (Bogert, Healy, Tukey \
                 1963). Structurally distinct from FFT band-energy: the cepstrum \
                 exposes periodicity / formant / pitch information that the power \
                 spectrum hides. Declared transform-law contract: FFT convention \
                 (DFT length + window) + log base + real-cepstrum vs complex-\
                 cepstrum + sampling rate. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MATCHED_FILTER_RESERVED_CANONICAL_ID),
            display_name: "Matched filter residual",
            motivation: "Cross-correlation with a declared template, residual = \
                 signal - best-template-match. Structurally distinct from any \
                 SEED spectral primitive (template-based vs basis-decomposition). \
                 Declared transform-law contract: template provenance (closed-\
                 form / pinned-fixture-hash / SEED-derived) + sampling-rate match \
                 between signal and template + normalization convention (peak / \
                 RMS / energy-normalized). Deterministic when template provenance \
                 is fixed; no learned templates without a separate canonical.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID),
            display_name: "Hilbert amplitude anomaly",
            motivation: "Analytic-signal extraction (Hilbert transform) and \
                 instantaneous-amplitude envelope. Structurally distinct from \
                 residual envelope exit at SEED id 22 (residual-magnitude-based \
                 envelope) because the analytic-signal envelope captures phase- \
                 coherent amplitude variation directly rather than via residual \
                 magnitude. Declared transform-law contract: analytic-signal \
                 extraction method (FFT-based with half-spectrum zeroing / filter-\
                 based with all-pass Hilbert filter) + sampling law. \
                 Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID),
            display_name: "FFT bandpower variant - parameterization shell",
            motivation: "FFT bandpower with declared specific band edges, window \
                 function, and normalization. The court rules: bandpower variants \
                 are ParameterizationOf(FFT band-energy, SEED id 12), NOT new \
                 canonical primitives. Appears in proposed_primitives but NOT in \
                 new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Wavelet family variant - parameterization shell",
            motivation: "Wavelet coefficient energy with declared specific wavelet \
                 family (Daubechies-N, Symlets, Coiflets, Haar) and decomposition \
                 level. The court rules: family / level variants are \
                 ParameterizationOf(wavelet coefficient energy, SEED id 39), NOT \
                 new canonical primitives.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "STFT window/hop variant - parameterization shell",
            motivation: "STFT ridge shift with declared specific window function \
                 (Hann / Hamming / Blackman) and hop fraction (25% / 50% / 75% \
                 overlap). The court rules: window / hop variants are \
                 ParameterizationOf(STFT ridge shift, T.12.e canonical 5503), \
                 NOT new canonical primitives.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Randomized spectral projection - rejected shell",
            motivation: "Random Fourier features (Rahimi & Recht 2007) / random \
                 spectral subspace approximations / randomized SVD-based spectral \
                 approximations are randomized in origin: at each invocation they \
                 sample a fresh random projection matrix. The court does NOT admit \
                 randomized spectral projection to the dedup-court delta's \
                 new_canonical_records. A future T.12.x proposal may admit a \
                 Deterministic_Spectral_Projection_Proxy canonical only if the \
                 sample seed, projection matrix definition (closed-form OR pinned-\
                 fixture-hash), projection dimension, and numeric mode are all \
                 brutally explicit. Until then this is a literature acknowledgement; \
                 the deterministic reduction is required and deliberately not \
                 provided in T.12.e.",
        },
    ]
}

/// Zero alias claims.
fn spectral_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Sixteen dedup-court decisions on the spectral batch.
fn spectral_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 6 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SPECTRAL_CENTROID_RESERVED_CANONICAL_ID),
            reason: "Spectral centroid shift: first-moment-of-power-spectrum \
                 decision functional. Declared transform law: power-spectrum \
                 convention + frequency-bin mapping + first-moment formula + \
                 sampling law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID),
            reason: "Wavelet packet energy: full packet-tree expansion (vs single-\
                 level wavelet coefficient energy at SEED 39). Declared transform \
                 law: wavelet family + packet-tree depth + energy convention + \
                 boundary handling + sampling law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(STFT_RIDGE_RESERVED_CANONICAL_ID),
            reason: "STFT ridge shift: short-time Fourier ridge tracking. Declared \
                 transform law: window function + window length + hop / overlap \
                 law + ridge selection law + extrapolation handling + sampling \
                 law. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CEPSTRAL_RESERVED_CANONICAL_ID),
            reason: "Cepstral anomaly: inverse FFT of log power spectrum. Declared \
                 transform law: FFT convention + log base + real-cepstrum vs \
                 complex-cepstrum + sampling rate. Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MATCHED_FILTER_RESERVED_CANONICAL_ID),
            reason: "Matched filter residual: cross-correlation with declared \
                 template. Declared transform law: template provenance (closed-\
                 form / pinned-fixture-hash / SEED-derived) + sampling-rate match \
                 + normalization convention (peak / RMS / energy-normalized). \
                 Deterministic.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID),
            reason: "Hilbert amplitude anomaly: analytic-signal instantaneous-\
                 amplitude envelope. Declared transform law: analytic-signal \
                 extraction method (FFT-based half-spectrum zeroing / filter-\
                 based all-pass Hilbert filter) + sampling law. Deterministic.",
        },
        // -- 5 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly stays canonical at SEED id 12. \
                 Declared transform law: window function + window length + FFT \
                 normalization + frequency-bin mapping + band definition + \
                 boundary handling + power / amplitude convention + sampling \
                 law. The SignalProcessing source class recognises FFT band-\
                 energy as the most fundamental power-spectrum-based primitive; \
                 no duplicate admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22. \
                 Declared transform law: envelope-extraction method + threshold + \
                 sampling law. No duplicate admitted; the Hilbert amplitude \
                 anomaly canonical (5506) uses a DIFFERENT extraction method \
                 (analytic signal vs residual magnitude) and is therefore a \
                 distinct primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            reason: "Spectral entropy stays canonical at SEED id 38. Declared \
                 transform law: bin definition + power-mass normalization \
                 (probability mass over bins) + log base + sampling law. No \
                 duplicate admitted; binning-variant aliases collapse to this \
                 record under ParameterizationOf if filed in a future T.12.x.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(WAVELET_COEFFICIENT_ENERGY_SEED_ID),
            reason: "Wavelet coefficient energy stays canonical at SEED id 39. \
                 Declared transform law: wavelet family + decomposition level + \
                 boundary handling + energy convention + sampling law. No \
                 duplicate admitted; family / level aliases collapse to \
                 ParameterizationOf (record 5508 below).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            reason: "Autocorrelation-coefficient break stays canonical at SEED id \
                 40. Declared transform law: lag set + normalization + window. \
                 No duplicate admitted.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy (SEED id 12) is the most fundamental power-\
                 spectrum-based primitive and the shared spectral-transform \
                 ancestor for the SignalProcessing source class. The court records \
                 the domain transfer without re-canonicalising FFT band-energy. \
                 Every other SEED spectral record above is recognised under \
                 ExistingCanonicalAuthorityResolution rather than \
                 DomainTransferOf to keep the wire-name set minimal.",
        },
        // -- 3 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID),
            reason: "FFT bandpower variant is ParameterizationOf(FFT band-energy \
                 anomaly, SEED id 12). The band-edge definition, window function \
                 (Hann / Hamming / Blackman / rectangular), and normalization \
                 (power-spectral-density / linear-magnitude / log-decibel) are \
                 the parameterization; the family-level decision functional is \
                 FFT band-energy's. The court declines to admit FFT bandpower \
                 variant as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID),
            reason: "Wavelet family variant is ParameterizationOf(wavelet \
                 coefficient energy, SEED id 39). The specific wavelet family \
                 (Daubechies-N for declared N / Symlets / Coiflets / Haar) and \
                 decomposition level are the parameterization; the family-level \
                 decision functional is wavelet coefficient energy's. The court \
                 declines to admit wavelet family variant as a new canonical \
                 primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID),
            reason: "STFT window/hop variant is ParameterizationOf(STFT ridge \
                 shift, T.12.e canonical 5503). The specific window function and \
                 hop fraction are the parameterization; the family-level decision \
                 functional is STFT ridge shift's. The court declines to admit \
                 STFT window/hop variant as a new canonical primitive.",
        },
        // -- 1 RejectedNotDeterministic record ---------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(RANDOMIZED_SPECTRAL_PROJECTION_RESERVED_PRIMITIVE_ID),
            reason: "Randomized spectral projection (Rahimi & Recht 2007 random \
                 Fourier features; random spectral subspace approximations; \
                 randomized SVD-based spectral approximations) is randomized in \
                 its literature definition: each invocation samples a fresh \
                 random projection matrix. Rejected as a literature-original \
                 canonical primitive for this deterministic corpus unless reduced \
                 to a declared deterministic proxy (sample seed, projection matrix \
                 definition - closed-form OR pinned-fixture-hash - projection \
                 dimension, and numeric mode all brutally explicit) in a later \
                 T.12.x proposal. Deterministic stance: the rejection is on the \
                 random sampling alone; the projection-then-FFT functional is \
                 deterministic given a fixed projection matrix.",
        },
    ]
}

/// Nine genealogy edges proposed for the post-freeze graph.
/// The six new canonicals carry DerivedFrom edges to the
/// shared spectral-transform ancestor (FFT band-energy, SEED
/// 12), plus a Generalizes edge for wavelet packet energy
/// over wavelet coefficient energy (its single-level ancestor).
/// The three parameterizations carry ParameterVariantOf edges.
fn spectral_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SPECTRAL_CENTROID_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(WAVELET_COEFFICIENT_ENERGY_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(STFT_RIDGE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CEPSTRAL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MATCHED_FILTER_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FFT_BANDPOWER_VARIANT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WAVELET_FAMILY_VARIANT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(WAVELET_COEFFICIENT_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(STFT_WINDOW_HOP_VARIANT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(STFT_RIDGE_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Seven source refs supporting the spectral expansion.
fn spectral_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "klapuri_spectral_centroid_2006",
            title: "Signal Processing Methods for Music Transcription (spectral centroid chapter)",
            year: 2006,
            venue: "Springer (engineering practice; spectral centroid as classical timbre statistic)",
        },
        ProposedSourceRef {
            citation_key: "wickerhauser_wavelet_packet_1994",
            title: "Adapted Wavelet Analysis from Theory to Software",
            year: 1994,
            venue: "A K Peters (wavelet packet decomposition chapter)",
        },
        ProposedSourceRef {
            citation_key: "portnoff_stft_1980",
            title: "Time-Frequency Representation of Digital Signals and Systems Based on Short-Time Fourier Analysis",
            year: 1980,
            venue: "IEEE Transactions on Acoustics, Speech, and Signal Processing 28(1)",
        },
        ProposedSourceRef {
            citation_key: "bogert_healy_tukey_cepstrum_1963",
            title: "The Quefrency Alanysis of Time Series for Echoes: Cepstrum, Pseudo-Autocovariance, Cross-Cepstrum and Saphe Cracking",
            year: 1963,
            venue: "Proceedings of the Symposium on Time Series Analysis (Wiley)",
        },
        ProposedSourceRef {
            citation_key: "turin_matched_filter_1960",
            title: "An Introduction to Matched Filters",
            year: 1960,
            venue: "IRE Transactions on Information Theory 6(3)",
        },
        ProposedSourceRef {
            citation_key: "huang_hilbert_amplitude_1998",
            title: "The Empirical Mode Decomposition and the Hilbert Spectrum for Nonlinear and Non-Stationary Time Series Analysis",
            year: 1998,
            venue: "Proceedings of the Royal Society A 454(1971)",
        },
        ProposedSourceRef {
            citation_key: "rahimi_recht_rff_2007",
            title: "Random Features for Large-Scale Kernel Machines (rejection-shell reference; random Fourier features are randomized)",
            year: 2007,
            venue: "NeurIPS 2007",
        },
    ]
}

/// Build the spectral `DedupCourtDelta`. `new_canonical_records`
/// admits the SIX genuinely new canonicals only; the three
/// parameterizations and the RANSAC-style rejection are
/// deliberately absent.
fn build_spectral_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_e_spectral_delta",
        vec![
            DetectorCanonicalId(SPECTRAL_CENTROID_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(WAVELET_PACKET_ENERGY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(STFT_RIDGE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CEPSTRAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MATCHED_FILTER_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(HILBERT_AMPLITUDE_RESERVED_CANONICAL_ID),
        ],
        Vec::<DetectorAliasId>::new(),
        Vec::<DetectorCanonicalId>::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    )
}

// ---------------------------------------------------------------
// Public seed entry point
// ---------------------------------------------------------------

/// Build the T.12.e spectral `CorpusAmendmentProposal`. Two
/// builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_e_spectral_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_e_spectral_first_proposal",
        "T.12.e files the Signal Processing / Spectral / Wavelet amendment \
         proposal. Adds six genuinely new canonical primitives (spectral centroid \
         shift, wavelet packet energy, STFT ridge shift, cepstral anomaly, \
         matched filter residual, Hilbert amplitude anomaly) at reserved canonical \
         ids 5501..=5506 with declared transform-law contracts (window function + \
         normalization + band / packet-tree depth + ridge selection law + FFT \
         convention + template provenance + analytic-signal extraction). Records \
         five ExistingCanonicalAuthorityResolution decisions keeping FFT band-\
         energy anomaly (SEED id 12), residual envelope exit (id 22), spectral \
         entropy (id 38), wavelet coefficient energy (id 39), autocorrelation-\
         coefficient break (id 40) canonical under the SignalProcessing source \
         class without duplication. Records one DomainTransferOf decision naming \
         FFT band-energy as the shared spectral-transform ancestor. Records three \
         ParameterizationOf decisions: FFT bandpower variant is ParameterizationOf \
         (FFT band-energy); wavelet family variant is ParameterizationOf(wavelet \
         coefficient energy); STFT window/hop variant is ParameterizationOf(STFT \
         ridge shift). Rejects randomized spectral projection (Rahimi & Recht \
         2007 random Fourier features and family) as RejectedNotDeterministic at \
         reserved id 5510 - randomized in origin; admission requires sample seed \
         + projection matrix definition + dimension + numeric mode declared. \
         Every record's reason text declares its specific transform-law contract \
         - the panel-locked warning was 'in spectral detectors, the transform law \
         is the detector'. Does NOT mutate SEED (SEED.len() stays at 54); status \
         = Open pending review.",
        SourceClass::SignalProcessing,
        build_spectral_expansion_batch(),
        build_spectral_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_e_spectral",
    )
}
