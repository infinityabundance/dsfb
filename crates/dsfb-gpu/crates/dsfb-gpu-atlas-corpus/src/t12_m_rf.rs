//! T.12.m — RF / Communications: the thirteenth real literature
//! expansion proposal filed through the T.12.0 amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.m files the RF / Communications amendment proposal.
//! > It admits only deterministic RF / communications signal
//! > witnesses whose signal representation, sampling law, unit
//! > law, carrier / channel assumption, synchronization
//! > assumption, window / transform law, decision functional,
//! > confuser profile, and numeric mode are declared; resolves
//! > SEED collisions; classifies variants as parameterizations
//! > or domain transfers; rejects learned RF fingerprinting
//! > classifiers and black-box modulation classifiers /
//! > proprietary spectrum-anomaly scores; and preserves the
//! > frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"An RF witness is
//! admissible only when the signal representation, sampling
//! law, unit law, carrier / channel assumption, synchronization
//! assumption, window / transform law, decision functional,
//! confuser profile, and numeric mode are declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.m admits deterministic RF / communications signal
//! > witnesses, not emitter attribution, transmitter
//! > identification, geolocation, spectrum-enforcement
//! > authority, military classification, or communications-
//! > intelligence conclusions.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.m's design began with a grep of [`crate::seed::SEED`]
//! for every RF candidate. The walk found **six** T.12.m-
//! relevant primitives already canonical:
//!
//! * **FFT band-energy anomaly** at SEED id 12 — RF spectral
//!   analysis is FFT-band-energy at its core.
//! * **Residual envelope exit** at SEED id 22 — RF analytic-
//!   envelope detection is residual-envelope-exit at its core.
//! * **Spectral entropy** at SEED id 38 — RF channel-occupancy
//!   and modulation-richness measures reduce to spectral
//!   entropy.
//! * **Autocorrelation break** at SEED id 40 — RF correlation-
//!   based detection (template / preamble correlation,
//!   cyclostationary feature extraction) reduces to or
//!   parameterizes the autocorrelation-break decision law.
//! * **Carrier-frequency-offset residual** at SEED id 53
//!   (Morelli & Mengali 1999 OFDM CFO estimator) — RF CFO
//!   detection IS this primitive: an MLE-based carrier-offset
//!   estimator with per-window residual threshold. The SEED
//!   record canonicalises CFO residual under RfCommunications.
//! * **Error Vector Magnitude (EVM) anomaly** at SEED id 54
//!   (Shafik / Rahman / Islam 2006 EVM-BER-SNR relations) —
//!   RF modulation-quality detection via rolling-EVM
//!   threshold over a declared symbol constellation IS this
//!   primitive; SEED 54 canonicalises it.
//!
//! All six become `ExistingCanonicalAuthorityResolution`
//! records under the `RfCommunications` source class. **Panel-
//! locked success-shape** (mirroring T.12.k / T.12.l): RF
//! reuses a lot of spectral, envelope, entropy, correlation,
//! carrier-offset, and modulation-quality structure; the
//! campaign's strength comes from cross-class dedup
//! discipline, not detector count.
//!
//! Six genuinely new canonicals at reserved ids 6303..=6308
//! survived the SEED-walk as structurally distinct decision
//! functionals. Reserved ids 6301 and 6302 are deliberately
//! unused in this band: the CFO and EVM ideas they once
//! shadowed collapsed onto SEED 53 and SEED 54 respectively
//! under the SEED-walk-first discipline; the court records
//! the collapse via ExistingCanonicalAuthorityResolution
//! rather than admitting new canonicals.
//!
//! * **Constellation spread witness** (6303) — declared symbol
//!   constellation, post-equalisation I/Q scatter, per-cluster
//!   second-moment computation, and spread decision law
//!   (cluster variance crosses threshold relative to nominal).
//!   Distinct from EVM because spread is a SECOND-MOMENT
//!   distribution claim, not a first-moment per-symbol error
//!   claim.
//! * **Channel impulse response (CIR) drift witness** (6304) —
//!   declared channel-sounding waveform, sampling law,
//!   tap-delay grid (multi-tap finite-impulse-response model),
//!   per-tap magnitude / phase, and CIR-drift decision law
//!   (per-tap magnitude shift relative to baseline crosses
//!   threshold; aggregate impulse-response-distance crosses
//!   threshold). Distinct from SEED 40 autocorrelation break
//!   because CIR is the SYSTEM RESPONSE to a declared
//!   impulse, not the autocorrelation of the observed signal
//!   itself.
//! * **IQ imbalance witness** (6305) — declared I/Q baseband
//!   representation, gain-balance and phase-balance estimators
//!   (per-axis gain ratio; per-axis phase offset), and IQ
//!   imbalance decision law (gain ratio departs from unity
//!   beyond threshold OR phase offset departs from 90° beyond
//!   threshold).
//! * **Phase-noise witness** (6306) — declared oscillator
//!   model (free-running / phase-locked-loop), phase-noise
//!   spectral density estimator (Welch on instantaneous phase
//!   residuals), per-offset-frequency phase-noise decision
//!   law (per-band phase-noise crosses spec mask in dBc/Hz).
//!   Razavi 1996 oscillator phase-noise canonical reference.
//! * **Symbol-timing offset residual witness** (6307) —
//!   declared symbol rate, Gardner / early-late timing-error
//!   detector, per-window timing-offset residual, and
//!   timing-offset decision law (residual crosses fraction of
//!   symbol period). Distinct from CFO because timing-offset
//!   is a SYMBOL-CLOCK alignment claim, not a CARRIER-PHASE
//!   alignment claim.
//! * **Cyclostationary feature shift witness** (6308) —
//!   declared cycle frequencies (symbol rate, carrier rate,
//!   chip rate per CDMA, OFDM subcarrier spacing), spectral-
//!   correlation-function estimator (Gardner 1987
//!   cyclostationary signal processing), per-cycle-frequency
//!   feature shift, and cyclostationary-feature decision law
//!   (per-cycle-frequency feature shift crosses threshold).
//!   Distinct from SEED 40 autocorrelation break because the
//!   cycle-frequency law is DECLARED, not implicit in a
//!   plain autocorrelation.
//!
//! Two domain transfers (panel-locked):
//!
//! * **FFT band-energy anomaly** (SEED 12) → `DomainTransferOf`
//!   for `RfCommunications` as the shared spectral RF ancestor
//!   (spectral mask violation 6309, SNR drop 6310 are RF
//!   descendants).
//! * **Residual envelope exit** (SEED 22) → `DomainTransferOf`
//!   for `RfCommunications` as the shared envelope-boundary
//!   ancestor (burst preamble miss 6311 RF descendant is an
//!   envelope decision on cross-correlation template
//!   residual).
//!
//! Four parameterizations (panel-candidate canonicals that
//! collapsed on closer inspection):
//!
//! * **Spectral mask violation** (6309) →
//!   `ParameterizationOf(FFT band-energy, SEED 12)` with
//!   declared regulatory spectral-mask envelope (ITU-R SM
//!   recommendations, ETSI EN, FCC Part 15 emission masks)
//!   on top of FFT band-energy semantics.
//! * **SNR drop** (6310) → `ParameterizationOf(FFT band-energy,
//!   SEED 12)` with declared signal-band / noise-band
//!   partition + signal-to-noise ratio law on top of FFT
//!   band-energy semantics.
//! * **Burst preamble miss** (6311) →
//!   `ParameterizationOf(Autocorrelation break, SEED 40)` with
//!   declared known preamble sequence + cross-correlation
//!   template + peak-detection threshold on top of correlation
//!   decision semantics.
//! * **Frame-error burst** (6312) → `ParameterizationOf(Error
//!   burst, SEED 41)` with declared RF frame format + CRC /
//!   forward-error-correction decode law + per-frame error
//!   indicator on top of generic error-burst semantics.
//!
//! Two rejections (seventh T.12.x with two
//! `RejectedNotDeterministic` records, following T.12.g, h, i,
//! j, k, l):
//!
//! * **Learned RF fingerprinting classifier** (6313) —
//!   `RejectedNotDeterministic`. RF fingerprinting via
//!   learned-embedding classifiers (Restuccia et al. 2019
//!   DeepRadioID; Sankhe et al. 2019 ORACLE; Wang et al. 2022
//!   RF-based device identification) claims transmitter
//!   identity attribution from opaque learned RF-feature
//!   embeddings. Admission requires a future T.12.x to admit
//!   a `Deterministic_RF_Fingerprint_Proxy` canonical with
//!   deterministic feature-extraction law, declared model or
//!   formula, declared tie-break law, declared numeric mode,
//!   no learned opaque embedding, and NO transmitter identity
//!   claim. The court does NOT issue emitter attribution.
//! * **Black-box modulation classifier / proprietary spectrum
//!   anomaly score** (6314) — `RejectedNotDeterministic`.
//!   Vendor RF intelligence (Keysight signal-analysis ML
//!   pipelines, Rohde & Schwarz spectrum monitoring AI, NI
//!   RFIC analyser ML, Ettus USRP-based learned pipelines)
//!   exposes modulation-classification and spectrum-anomaly
//!   scores without a public deterministic feature-extraction
//!   law, declared training-data anchor, declared tie-break
//!   law, or declared numeric mode. Admission requires a
//!   future T.12.x to admit a
//!   `Deterministic_Modulation_Classifier_Proxy` canonical
//!   with deterministic feature-extraction law + declared
//!   formula + training-data anchor + feature schema + tie-
//!   break + numeric mode all brutally explicit. The court
//!   does NOT issue spectrum-enforcement authority.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×6 (6303..=6308).
//! * `ExistingCanonicalAuthorityResolution` ×6 — SEED 12, 22,
//!   38, 40, 53, 54.
//! * `DomainTransferOf` ×2 — SEED 12 (spectral ancestor) +
//!   SEED 22 (envelope ancestor).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 6 + 6 + 2 + 4 + 2 = **20 dedup-court records**.
//!
//! ## Emitter-attribution / geolocation / spectrum-enforcement discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND
//! ExistingCanonicalAuthorityResolution reason text MUST
//! describe its record as a "RF signal witness" / "spectral
//! witness" / "channel witness" / "modulation-quality witness"
//! / "synchronization witness" — NEVER as an emitter
//! attribution, transmitter identification, geolocation,
//! spectrum-enforcement, military-classification, or
//! communications-intelligence claim. The dedicated load-
//! bearing negatives scan every such reason for forbidden
//! terms (emitter attribution, emitter identification,
//! transmitter identification, transmitter identity, device
//! identification, transmitter fingerprint, geolocation
//! certainty, geolocates the, transmitter location, spectrum
//! enforcement, regulatory enforcement, illegal transmission,
//! unauthorized transmission, military classification,
//! signals intelligence, comint conclusion, sigint verdict)
//! and assert every qualifying reason ends with the panel-
//! locked non-claim "RF signal witness, not emitter
//! attribution or spectrum enforcement". Forbidden terms
//! appear ONLY in `RejectedNotDeterministic` reason text.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.m RF / communications
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
// Reserved id constants (panel-locked; 6303..=6314 used in this
// proposal; 6301 + 6302 deliberately unused in this band — the
// CFO and EVM ideas they once shadowed collapsed onto SEED 53
// and SEED 54 respectively under the SEED-walk-first discipline;
// 6315..=6399 reserved for future RF / Communications proposals)
// ---------------------------------------------------------------

/// Reserved canonical id for Constellation spread witness.
/// Second-moment scatter measurement; distinct from EVM
/// which is a first-moment per-symbol error.
pub const CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID: u32 = 6303;

/// Reserved canonical id for Channel impulse response (CIR)
/// drift witness. Distinct from SEED 40 autocorrelation
/// break because CIR is the SYSTEM response to a declared
/// impulse, not autocorrelation of the observed signal.
pub const CIR_DRIFT_RESERVED_CANONICAL_ID: u32 = 6304;

/// Reserved canonical id for IQ imbalance witness. Per-axis
/// gain and phase imbalance measurement on a declared baseband
/// I/Q representation.
pub const IQ_IMBALANCE_RESERVED_CANONICAL_ID: u32 = 6305;

/// Reserved canonical id for Phase-noise witness. Per-offset-
/// frequency phase-noise spectral density against a declared
/// oscillator model. Razavi 1996 canonical reference.
pub const PHASE_NOISE_RESERVED_CANONICAL_ID: u32 = 6306;

/// Reserved canonical id for Symbol-timing offset residual
/// witness. Gardner / early-late timing-error detector law.
/// Distinct from CFO because it is a SYMBOL-CLOCK alignment
/// claim, not a CARRIER-PHASE alignment claim.
pub const SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID: u32 = 6307;

/// Reserved canonical id for Cyclostationary feature shift
/// witness. Gardner 1987 cyclostationary signal processing.
/// Distinct from SEED 40 autocorrelation break because the
/// cycle-frequency law is DECLARED, not implicit.
pub const CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID: u32 = 6308;

/// Reserved id for Spectral mask violation.
/// `ParameterizationOf(FFT band-energy, SEED 12)`.
pub const SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID: u32 = 6309;

/// Reserved id for SNR drop. `ParameterizationOf(FFT band-
/// energy, SEED 12)`.
pub const SNR_DROP_RESERVED_PRIMITIVE_ID: u32 = 6310;

/// Reserved id for Burst preamble miss.
/// `ParameterizationOf(Autocorrelation break, SEED 40)`.
pub const BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID: u32 = 6311;

/// Reserved id for Frame-error burst. `ParameterizationOf
/// (Error burst, SEED 41)`.
pub const FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID: u32 = 6312;

/// Reserved id for Learned RF fingerprinting classifier.
/// `RejectedNotDeterministic`.
pub const LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID: u32 = 6313;

/// Reserved id for Black-box modulation classifier /
/// proprietary spectrum anomaly score.
/// `RejectedNotDeterministic`.
pub const BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID: u32 = 6314;

// Existing SEED canonical ids referenced by T.12.m.

/// FFT band-energy anomaly — SEED canonical id 12. Shared
/// spectral RF ancestor.
pub const FFT_BAND_ENERGY_SEED_ID: u32 = 12;

/// Residual envelope exit — SEED canonical id 22. Shared
/// envelope-boundary RF ancestor.
pub const RESIDUAL_ENVELOPE_EXIT_SEED_ID: u32 = 22;

/// Spectral entropy — SEED canonical id 38. RF channel-
/// occupancy / modulation-richness ancestor.
pub const SPECTRAL_ENTROPY_SEED_ID: u32 = 38;

/// Autocorrelation break — SEED canonical id 40. RF
/// correlation-based detection ancestor (template / preamble
/// correlation; cyclostationary feature extraction).
pub const AUTOCORRELATION_BREAK_SEED_ID: u32 = 40;

/// Error burst — SEED canonical id 41. Generic error-burst
/// ancestor for frame-error parameterization.
pub const ERROR_BURST_SEED_ID: u32 = 41;

/// Carrier-frequency-offset residual — SEED canonical id 53
/// (Morelli & Mengali 1999 OFDM CFO estimator). RF CFO
/// detection canonical at SEED level; T.12.m records an
/// `ExistingCanonicalAuthorityResolution` rather than adding
/// a new canonical.
pub const CFO_RESIDUAL_SEED_ID: u32 = 53;

/// Error Vector Magnitude (EVM) anomaly — SEED canonical id 54
/// (Shafik / Rahman / Islam 2006 EVM-BER-SNR relations). RF
/// modulation-quality detection canonical at SEED level;
/// T.12.m records an `ExistingCanonicalAuthorityResolution`
/// rather than adding a new canonical.
pub const EVM_ANOMALY_SEED_ID: u32 = 54;

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
// Builders for the RF / communications expansion batch
// ---------------------------------------------------------------

/// Build the RF / communications `CorpusExpansionBatch` body.
fn build_rf_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_m_rf_first_proposal",
        SourceClass::RfCommunications,
        rf_proposed_primitives(),
        rf_proposed_aliases(),
        rf_proposed_dedup_records(),
        rf_proposed_genealogy_edges(),
        rf_proposed_source_refs(),
    )
}

/// Twelve proposed primitives: 6 canonical + 4 parameterization
/// shells + 2 rejection shells. The "tight canonical set, heavy
/// spectral / envelope / entropy / correlation / carrier-offset
/// / modulation-quality authority resolution, clear rejection of
/// learned RF fingerprinting" shape applies the panel-locked
/// T.12.k / T.12.l success posture to RF. Reserved ids 6301 and
/// 6302 are deliberately unused — the CFO and EVM ideas they
/// once shadowed collapsed onto SEED 53 and SEED 54 respectively
/// under the SEED-walk-first discipline; the court records the
/// collapse via `ExistingCanonicalAuthorityResolution`.
fn rf_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID),
            display_name: "Constellation spread witness",
            motivation: "Constellation spread RF signal witness. Required \
                 contract: signal representation (post-equalisation I/Q \
                 baseband), sampling law, unit law, carrier assumption, \
                 synchronization assumption, declared symbol constellation, \
                 window / transform law (per-cluster second-moment \
                 computation; cluster assignment by nearest-ideal-point), \
                 decision functional (per-cluster variance crosses \
                 threshold relative to nominal AWGN spread), confuser \
                 profile (AWGN, multipath fading, equaliser convergence), \
                 numeric mode. Structurally distinct from SEED 54 EVM \
                 because constellation spread is a SECOND-MOMENT \
                 distribution claim per cluster, not a first-moment per- \
                 symbol error claim. RF modulation-quality signal witness, \
                 not emitter attribution or spectrum enforcement.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CIR_DRIFT_RESERVED_CANONICAL_ID),
            display_name: "Channel impulse response (CIR) drift witness",
            motivation: "Channel impulse response RF channel-witness. \
                 Required contract: signal representation (baseband I/Q \
                 with declared channel-sounding waveform — pilot symbols, \
                 PN sequence, or OFDM channel-estimation pilots), sampling \
                 law, unit law, channel assumption (declared multipath \
                 finite-impulse-response model with tap-delay grid), \
                 synchronization assumption, window / transform law (per- \
                 sounding-window estimated CIR — per-tap magnitude and \
                 phase), decision functional (per-tap magnitude shift \
                 relative to baseline CIR crosses threshold; aggregate \
                 impulse-response-distance crosses threshold), confuser \
                 profile (mobility-induced Doppler, equaliser adaptation, \
                 timing jitter), numeric mode. Structurally distinct from \
                 SEED 40 autocorrelation break because CIR is the SYSTEM \
                 RESPONSE to a declared impulse, not the autocorrelation \
                 of the observed signal itself. RF channel signal witness, \
                 not emitter attribution or geolocation.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(IQ_IMBALANCE_RESERVED_CANONICAL_ID),
            display_name: "IQ imbalance witness",
            motivation: "IQ imbalance RF signal witness. Required \
                 contract: signal representation (baseband I/Q at declared \
                 sample rate), sampling law, unit law (gain ratio \
                 dimensionless; phase offset in radians or degrees), \
                 carrier assumption, synchronization assumption, window / \
                 transform law (per-window gain-balance estimator — per- \
                 axis gain ratio; per-window phase-balance estimator — \
                 per-axis phase offset from 90°), decision functional \
                 (gain ratio departs from unity beyond threshold OR phase \
                 offset departs from 90° beyond threshold), confuser \
                 profile (DC offset, ADC bit-depth quantization, low-IF \
                 image leakage), numeric mode. RF modulation-quality \
                 signal witness, not emitter attribution or transmitter \
                 fingerprint.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PHASE_NOISE_RESERVED_CANONICAL_ID),
            display_name: "Phase-noise witness",
            motivation: "Phase-noise RF signal witness (Razavi 1996 \
                 oscillator phase-noise canonical reference). Required \
                 contract: signal representation (baseband I/Q with \
                 unwrapped instantaneous-phase extraction), sampling law, \
                 unit law (dBc/Hz at declared offset frequencies), \
                 carrier assumption, declared oscillator model (free- \
                 running / phase-locked-loop with declared loop \
                 bandwidth), window / transform law (Welch periodogram on \
                 instantaneous phase residuals at declared offset \
                 frequencies — e.g. 1 kHz / 10 kHz / 100 kHz / 1 MHz), \
                 decision functional (per-offset-frequency phase-noise \
                 spectral density crosses spec mask in dBc/Hz), confuser \
                 profile (mechanical vibration, supply noise, thermal \
                 fluctuation, mixer leakage), numeric mode. RF modulation- \
                 quality signal witness, not emitter attribution or \
                 transmitter fingerprint.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID),
            display_name: "Symbol-timing offset residual witness",
            motivation: "Symbol-timing offset RF synchronization signal \
                 witness. Required contract: signal representation \
                 (baseband I/Q), sampling law (oversampling factor relative \
                 to declared symbol rate), unit law (fraction of symbol \
                 period), carrier assumption, declared symbol rate, \
                 synchronization assumption (Gardner / early-late timing- \
                 error detector or Mueller-Müller detector), window / \
                 transform law (per-window timing-offset residual \
                 estimator), decision functional (timing-offset residual \
                 crosses declared fraction of symbol period — e.g. ±10% of \
                 symbol period), confuser profile (clock drift, multipath \
                 self-interference, pulse-shape mismatch), numeric mode. \
                 Structurally distinct from CFO (6301) because timing- \
                 offset is a SYMBOL-CLOCK alignment claim, not a CARRIER- \
                 PHASE alignment claim. RF synchronization signal witness, \
                 not emitter attribution or spectrum enforcement.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID,
            ),
            display_name: "Cyclostationary feature shift witness",
            motivation: "Cyclostationary feature shift RF signal witness \
                 (Gardner 1987 cyclostationary signal processing). \
                 Required contract: signal representation (baseband or \
                 bandpass), sampling law, unit law, carrier assumption, \
                 declared cycle frequencies (symbol rate, carrier rate, \
                 chip rate per CDMA, OFDM subcarrier spacing), \
                 synchronization assumption, window / transform law \
                 (spectral correlation function estimator — time-smoothed \
                 cyclic periodogram or FFT-accumulation method over \
                 declared cycle frequencies), decision functional (per- \
                 cycle-frequency feature shift crosses threshold relative \
                 to nominal), confuser profile (AWGN, non-cyclostationary \
                 interferers, frequency-shifted images), numeric mode. \
                 Structurally distinct from SEED 40 autocorrelation break \
                 because the cycle-frequency law is DECLARED, not \
                 implicit in a plain autocorrelation. RF spectral signal \
                 witness, not emitter attribution or spectrum enforcement.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Spectral mask violation - parameterization shell",
            motivation: "Regulatory-mask parameterization of FFT band- \
                 energy (SEED id 12) with declared regulatory spectral- \
                 mask envelope (ITU-R SM recommendations, ETSI EN, FCC \
                 Part 15 emission masks) + per-band-power-vs-mask decision \
                 law on top of FFT band-energy semantics. The court rules: \
                 spectral mask violation is ParameterizationOf(FFT band- \
                 energy, SEED 12), NOT a new canonical primitive. Appears \
                 in proposed_primitives but NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SNR_DROP_RESERVED_PRIMITIVE_ID),
            display_name: "SNR drop - parameterization shell",
            motivation: "Signal-to-noise-ratio parameterization of FFT \
                 band-energy (SEED id 12) with declared signal-band / \
                 noise-band partition + per-window SNR computation + SNR- \
                 shift decision law on top of FFT band-energy semantics. \
                 The court rules: SNR drop is ParameterizationOf(FFT band- \
                 energy, SEED 12), NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID),
            display_name: "Burst preamble miss - parameterization shell",
            motivation: "Cross-correlation template parameterization of \
                 Autocorrelation break (SEED id 40) with declared known \
                 preamble sequence + cross-correlation template (matched- \
                 filter over preamble) + peak-detection threshold + peak- \
                 absence decision law on top of correlation decision \
                 semantics. The court rules: burst preamble miss is \
                 ParameterizationOf(Autocorrelation break, SEED 40), NOT a \
                 new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID),
            display_name: "Frame-error burst - parameterization shell",
            motivation: "RF-frame parameterization of Error burst (SEED id \
                 41) with declared RF frame format (IEEE 802.11 / IEEE \
                 802.15.4 / 3GPP LTE / 5G NR frame structure) + CRC / \
                 forward-error-correction decode law + per-frame error \
                 indicator + per-window frame-error rate + frame-error \
                 burst decision law on top of generic error-burst \
                 semantics. The court rules: frame-error burst is \
                 ParameterizationOf(Error burst, SEED 41), NOT a new \
                 canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Learned RF fingerprinting classifier - rejected shell",
            motivation: "Learned RF fingerprinting classifiers (Restuccia \
                 et al. 2019 DeepRadioID; Sankhe et al. 2019 ORACLE; Wang \
                 et al. 2022 RF-based device identification; deep CNN / \
                 LSTM / transformer pipelines over baseband I/Q) claim \
                 transmitter identity attribution from opaque learned RF- \
                 feature embeddings without a deterministic feature- \
                 extraction law, declared formula, declared tie-break law, \
                 or declared numeric mode. The court does NOT admit \
                 learned RF fingerprinting classifiers to the dedup-court \
                 delta's new_canonical_records. A future T.12.x may admit \
                 a Deterministic_RF_Fingerprint_Proxy canonical only if a \
                 deterministic feature-extraction law, declared model or \
                 formula, declared tie-break law, declared numeric mode, \
                 no learned opaque embedding, and NO transmitter identity \
                 claim are all brutally explicit. The court does NOT \
                 issue emitter attribution, transmitter identification, \
                 or transmitter fingerprint; those terms appear here only \
                 to describe what is NOT admitted.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(
                BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            display_name: "Black-box modulation classifier / proprietary spectrum anomaly score - rejected shell",
            motivation: "Vendor RF intelligence (Keysight signal-analysis \
                 ML pipelines, Rohde & Schwarz spectrum monitoring AI, NI \
                 RFIC analyser ML, Ettus USRP-based learned pipelines) \
                 exposes modulation-classification and spectrum-anomaly \
                 scores without a public deterministic feature-extraction \
                 law, declared training-data anchor, declared tie-break \
                 law, or declared numeric mode. The court does NOT admit \
                 these to new_canonical_records. A future T.12.x may admit \
                 a Deterministic_Modulation_Classifier_Proxy canonical \
                 only if a deterministic feature-extraction law + declared \
                 formula + training-data anchor + feature schema + tie- \
                 break + numeric mode are all brutally explicit. The \
                 court does NOT issue spectrum-enforcement authority, \
                 regulatory enforcement, or unauthorized-transmission \
                 verdicts; those terms appear here only to describe what \
                 is NOT admitted.",
        },
    ]
}

/// Zero alias claims (T.12.m routes everything through dedup
/// records and existing-canonical authority resolutions).
fn rf_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Twenty dedup-court decisions on the RF / communications batch.
fn rf_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 6 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID),
            reason: "Constellation spread witness: declared signal \
                 representation (post-equalisation I/Q) + sampling law + \
                 unit law + carrier assumption + synchronization \
                 assumption + declared symbol constellation + window / \
                 transform law (per-cluster second-moment computation; \
                 cluster assignment by nearest-ideal-point) + decision \
                 functional (per-cluster variance crosses threshold \
                 relative to nominal AWGN spread) + confuser profile \
                 (AWGN, multipath fading, equaliser convergence) + \
                 numeric mode. Structurally distinct from SEED 54 EVM \
                 because spread is a SECOND-MOMENT distribution claim per \
                 cluster, not a first-moment per-symbol error claim. RF \
                 modulation-quality signal witness, not emitter \
                 attribution or spectrum enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CIR_DRIFT_RESERVED_CANONICAL_ID),
            reason: "Channel impulse response (CIR) drift witness: \
                 declared signal representation (baseband I/Q with \
                 declared channel-sounding waveform — pilot symbols, PN \
                 sequence, or OFDM channel-estimation pilots) + sampling \
                 law + unit law + channel assumption (declared multipath \
                 finite-impulse-response model with tap-delay grid) + \
                 synchronization assumption + window / transform law \
                 (per-sounding-window estimated CIR — per-tap magnitude \
                 and phase) + decision functional (per-tap magnitude \
                 shift relative to baseline CIR crosses threshold; \
                 aggregate impulse-response-distance crosses threshold) + \
                 confuser profile (mobility-induced Doppler, equaliser \
                 adaptation, timing jitter) + numeric mode. Structurally \
                 distinct from SEED 40 autocorrelation break because CIR \
                 is the SYSTEM RESPONSE to a declared impulse, not the \
                 autocorrelation of the observed signal. RF channel \
                 signal witness, not emitter attribution or geolocation.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(IQ_IMBALANCE_RESERVED_CANONICAL_ID),
            reason: "IQ imbalance witness: declared signal representation \
                 (baseband I/Q at declared sample rate) + sampling law + \
                 unit law (gain ratio dimensionless; phase offset in \
                 radians or degrees) + carrier assumption + \
                 synchronization assumption + window / transform law \
                 (per-window gain-balance estimator and per-window phase- \
                 balance estimator) + decision functional (gain ratio \
                 departs from unity beyond threshold OR phase offset \
                 departs from 90° beyond threshold) + confuser profile \
                 (DC offset, ADC bit-depth quantization, low-IF image \
                 leakage) + numeric mode. RF modulation-quality signal \
                 witness, not emitter attribution or transmitter \
                 fingerprint.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PHASE_NOISE_RESERVED_CANONICAL_ID),
            reason: "Phase-noise witness (Razavi 1996): declared signal \
                 representation (baseband I/Q with unwrapped instantaneous- \
                 phase extraction) + sampling law + unit law (dBc/Hz at \
                 declared offset frequencies) + carrier assumption + \
                 declared oscillator model (free-running or phase-locked- \
                 loop with declared loop bandwidth) + window / transform \
                 law (Welch periodogram on instantaneous phase residuals \
                 at declared offset frequencies, e.g. 1 kHz / 10 kHz / \
                 100 kHz / 1 MHz) + decision functional (per-offset- \
                 frequency phase-noise spectral density crosses spec mask \
                 in dBc/Hz) + confuser profile (mechanical vibration, \
                 supply noise, thermal fluctuation, mixer leakage) + \
                 numeric mode. RF modulation-quality signal witness, not \
                 emitter attribution or transmitter fingerprint.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID),
            reason: "Symbol-timing offset residual witness: declared \
                 signal representation (baseband I/Q) + sampling law \
                 (oversampling factor relative to declared symbol rate) + \
                 unit law (fraction of symbol period) + carrier \
                 assumption + declared symbol rate + synchronization \
                 assumption (Gardner / early-late timing-error detector \
                 or Mueller-Müller detector) + window / transform law \
                 (per-window timing-offset residual estimator) + decision \
                 functional (timing-offset residual crosses declared \
                 fraction of symbol period, e.g. ±10%) + confuser profile \
                 (clock drift, multipath self-interference, pulse-shape \
                 mismatch) + numeric mode. Structurally distinct from \
                 CFO (6301) because timing-offset is a SYMBOL-CLOCK \
                 alignment claim, not a CARRIER-PHASE alignment claim. \
                 RF synchronization signal witness, not emitter \
                 attribution or spectrum enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Cyclostationary feature shift witness (Gardner 1987): \
                 declared signal representation (baseband or bandpass) + \
                 sampling law + unit law + carrier assumption + declared \
                 cycle frequencies (symbol rate, carrier rate, chip rate \
                 per CDMA, OFDM subcarrier spacing) + synchronization \
                 assumption + window / transform law (spectral correlation \
                 function estimator — time-smoothed cyclic periodogram or \
                 FFT-accumulation method over declared cycle frequencies) \
                 + decision functional (per-cycle-frequency feature shift \
                 crosses threshold relative to nominal) + confuser \
                 profile (AWGN, non-cyclostationary interferers, \
                 frequency-shifted images) + numeric mode. Structurally \
                 distinct from SEED 40 autocorrelation break because the \
                 cycle-frequency law is DECLARED, not implicit in a plain \
                 autocorrelation. RF spectral signal witness, not emitter \
                 attribution or spectrum enforcement.",
        },
        // -- 6 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly stays canonical at SEED id \
                 12 under RfCommunications. RF spectral analysis is FFT- \
                 band-energy at its core. Declared signal representation \
                 (baseband or bandpass) + sampling law + carrier \
                 assumption + window / transform law (FFT size, window \
                 function, overlap) + per-band power computation + \
                 decision law (per-band power crosses baseline-derived \
                 threshold) + unit law + numeric mode. No duplicate \
                 admitted; spectral mask violation (6309) and SNR drop \
                 (6310) collapse here as ParameterizationOf. RF spectral \
                 signal witness, not emitter attribution or spectrum \
                 enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit stays canonical at SEED id 22 \
                 under RfCommunications. RF analytic-envelope detection \
                 (analytic-signal magnitude via Hilbert transform; \
                 amplitude-modulated envelope detection) reduces to \
                 envelope-residual exit semantics. Declared signal \
                 representation + sampling law + residual definition + \
                 nominal envelope bounds + envelope-exit decision law + \
                 unit law + numeric mode. No duplicate admitted; burst \
                 preamble miss (6311) is ParameterizationOf SEED 40, not \
                 here, but the envelope-boundary semantic is reused for \
                 RF envelope detectors generally. RF envelope signal \
                 witness, not emitter attribution or spectrum \
                 enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            reason: "Spectral entropy stays canonical at SEED id 38 under \
                 RfCommunications. RF channel-occupancy and modulation- \
                 richness measures reduce to spectral entropy (Shannon \
                 entropy of normalised per-bin power spectrum). Declared \
                 signal representation + sampling law + window / \
                 transform law + bin normalisation + entropy computation \
                 + decision law (per-window spectral entropy crosses \
                 threshold relative to baseline) + unit law (nats or \
                 bits) + numeric mode. No duplicate admitted. RF spectral \
                 signal witness, not emitter attribution or spectrum \
                 enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            reason: "Autocorrelation break stays canonical at SEED id 40 \
                 under RfCommunications. RF correlation-based detection \
                 (template / preamble correlation; cyclostationary \
                 feature extraction generalisation) reduces to or \
                 parameterizes the autocorrelation-break decision law. \
                 Declared signal representation + sampling law + lag \
                 grid + autocorrelation computation + decision law \
                 (per-lag autocorrelation departs from baseline beyond \
                 threshold) + unit law + numeric mode. No duplicate \
                 admitted; burst preamble miss (6311) collapses here as \
                 ParameterizationOf with declared known-preamble cross- \
                 correlation template. RF correlation signal witness, \
                 not emitter attribution or spectrum enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(CFO_RESIDUAL_SEED_ID),
            reason: "Carrier-frequency-offset residual stays canonical at \
                 SEED id 53 under RfCommunications (Morelli & Mengali \
                 1999 OFDM CFO estimator). Declared signal representation \
                 (baseband I/Q at declared sample rate) + sampling law + \
                 unit law (Hz / radians per sample) + carrier assumption \
                 (declared expected carrier frequency) + synchronization \
                 assumption (preamble or pilot location) + window / \
                 transform law (per-window MLE frequency estimator over \
                 preamble) + decision functional (per-window CFO \
                 residual crosses tolerance derived from symbol rate) + \
                 confuser profile (Doppler shift, phase noise, oscillator \
                 drift) + numeric mode. No duplicate admitted; the SEED- \
                 walk-first discipline declined to admit a new canonical \
                 6301 because SEED 53 already canonicalises CFO residual \
                 detection. Reserved id 6301 stays unused in this band. \
                 RF signal witness, not emitter attribution or spectrum \
                 enforcement.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(EVM_ANOMALY_SEED_ID),
            reason: "Error Vector Magnitude (EVM) anomaly stays canonical \
                 at SEED id 54 under RfCommunications (Shafik / Rahman / \
                 Islam 2006 EVM-BER-SNR relations). Declared signal \
                 representation (baseband I/Q) + sampling law + unit law \
                 (dimensionless ratio or dB / % RMS) + carrier assumption \
                 + synchronization assumption (declared symbol-timing \
                 recovery law + carrier-phase recovery law) + declared \
                 symbol constellation (BPSK / QPSK / 16-QAM / 64-QAM / \
                 256-QAM) + window / transform law (per-symbol error \
                 vector — received-symbol minus nearest-ideal-constellation- \
                 point) + decision functional (rolling-EVM threshold over \
                 declared window crosses limit) + confuser profile \
                 (frequency-selective channel, IQ imbalance, phase noise) \
                 + numeric mode. No duplicate admitted; the SEED-walk- \
                 first discipline declined to admit a new canonical 6302 \
                 because SEED 54 already canonicalises EVM anomaly \
                 detection. Reserved id 6302 stays unused in this band. \
                 Constellation spread (6303) and IQ imbalance (6305) \
                 carry DerivedFrom genealogy edges to SEED 54. RF \
                 modulation-quality signal witness, not emitter \
                 attribution or spectrum enforcement.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            reason: "FFT band-energy anomaly (SEED id 12) is the shared \
                 spectral ancestor for the RfCommunications source class. \
                 Spectral mask violation (6309) and SNR drop (6310) are \
                 RF descendants. The court records the domain transfer \
                 without re-canonicalising FFT band-energy.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(RESIDUAL_ENVELOPE_EXIT_SEED_ID),
            reason: "Residual envelope exit (SEED id 22) is the shared \
                 envelope-boundary ancestor for the RfCommunications \
                 source class. RF analytic-signal envelope detection and \
                 amplitude-modulated envelope detection inherit the \
                 envelope-exit semantic without re-canonicalisation. The \
                 court records the domain transfer without re- \
                 canonicalising Residual envelope exit.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID),
            reason: "Spectral mask violation is ParameterizationOf(FFT \
                 band-energy, SEED id 12). Regulatory-mask \
                 parameterization with declared spectral-mask envelope \
                 (ITU-R SM recommendations, ETSI EN, FCC Part 15 \
                 emission masks) + per-band-power-vs-mask decision law. \
                 The court declines to admit spectral mask violation as \
                 a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(SNR_DROP_RESERVED_PRIMITIVE_ID),
            reason: "SNR drop is ParameterizationOf(FFT band-energy, SEED \
                 id 12). Signal-to-noise-ratio parameterization with \
                 declared signal-band / noise-band partition + per-window \
                 SNR computation + SNR-shift decision law. The court \
                 declines to admit SNR drop as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID),
            reason: "Burst preamble miss is ParameterizationOf \
                 (Autocorrelation break, SEED id 40). Cross-correlation \
                 template parameterization with declared known preamble \
                 sequence + cross-correlation template (matched-filter \
                 over preamble) + peak-detection threshold + peak- \
                 absence decision law. The court declines to admit burst \
                 preamble miss as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID),
            reason: "Frame-error burst is ParameterizationOf(Error burst, \
                 SEED id 41). RF-frame parameterization with declared RF \
                 frame format (IEEE 802.11 / IEEE 802.15.4 / 3GPP LTE / \
                 5G NR frame structure) + CRC / forward-error-correction \
                 decode law + per-frame error indicator + per-window \
                 frame-error rate + frame-error burst decision law. The \
                 court declines to admit frame-error burst as a new \
                 canonical primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID),
            reason: "Learned RF fingerprinting classifier (Restuccia et \
                 al. 2019 DeepRadioID; Sankhe et al. 2019 ORACLE; Wang \
                 et al. 2022 RF-based device identification; deep CNN / \
                 LSTM / transformer pipelines over baseband I/Q) claims \
                 emitter attribution and transmitter identification from \
                 opaque learned RF-feature embeddings. Rejected unless \
                 reduced to a declared Deterministic_RF_Fingerprint_Proxy \
                 with deterministic feature-extraction law, declared \
                 formula, declared tie-break law, declared numeric mode, \
                 no learned opaque embedding, and no transmitter identity \
                 claim, all brutally explicit in a later T.12.x. The \
                 court does NOT issue emitter attribution, transmitter \
                 identification, transmitter fingerprint, or geolocation \
                 certainty; those terms appear here only to describe what \
                 is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(
                BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID,
            ),
            reason: "Black-box modulation classifier / proprietary \
                 spectrum anomaly score (Keysight signal-analysis ML \
                 pipelines, Rohde & Schwarz spectrum monitoring AI, NI \
                 RFIC analyser ML, Ettus USRP-based learned pipelines) \
                 exposes modulation-classification and spectrum-anomaly \
                 scores without a deterministic feature-extraction law, \
                 declared formula, training-data anchor, feature schema, \
                 tie-break, or numeric mode. Rejected unless reduced to \
                 a Deterministic_Modulation_Classifier_Proxy with all \
                 six contract fields brutally explicit in a later \
                 T.12.x. The court does NOT issue spectrum-enforcement \
                 authority, regulatory enforcement, illegal-transmission \
                 verdicts, or unauthorized-transmission verdicts; those \
                 terms appear here only to describe what is NOT admitted.",
        },
    ]
}

/// Ten genealogy edges proposed for the post-freeze graph.
fn rf_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(EVM_ANOMALY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CIR_DRIFT_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(IQ_IMBALANCE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(EVM_ANOMALY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(PHASE_NOISE_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(
                CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID,
            ),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SNR_DROP_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FFT_BAND_ENERGY_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(AUTOCORRELATION_BREAK_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(ERROR_BURST_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the RF / communications expansion.
fn rf_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "proakis_digital_communications_2008",
            title: "Digital Communications",
            year: 2008,
            venue: "McGraw-Hill 5th ed. (digital communications canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "sklar_digital_communications_2001",
            title: "Digital Communications: Fundamentals and Applications",
            year: 2001,
            venue: "Prentice Hall 2nd ed. (digital communications canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "gardner_cyclostationary_1987",
            title: "Statistical Spectral Analysis: A Non-Probabilistic Theory",
            year: 1987,
            venue: "Prentice Hall (cyclostationary signal processing canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "razavi_rf_microelectronics_1996",
            title: "RF Microelectronics",
            year: 1996,
            venue: "Prentice Hall (phase-noise canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "rappaport_wireless_2014",
            title: "Wireless Communications: Principles and Practice",
            year: 2014,
            venue: "Prentice Hall 2nd ed. (wireless channel canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "gardner_timing_recovery_1986",
            title: "A BPSK/QPSK Timing-Error Detector for Sampled Receivers",
            year: 1986,
            venue: "IEEE Transactions on Communications COM-34(5) (symbol-timing \
                error detector canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "ieee_802_11_frame",
            title: "IEEE Standard for Information Technology --- Telecommunications \
                and Information Exchange between Systems --- Local and Metropolitan \
                Area Networks --- Specific Requirements --- Part 11",
            year: 2020,
            venue: "IEEE Std 802.11 (preamble / frame format canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "itu_r_sm_spectral_masks",
            title: "ITU-R SM Recommendations on Spectrum Management and Emission \
                Masks",
            year: 2023,
            venue: "ITU-R Study Group 1 (cited only to anchor spectral-mask \
                vocabulary, not to claim regulatory compliance)",
        },
        ProposedSourceRef {
            citation_key: "vendor_rf_intelligence_refs",
            title: "Vendor RF Intelligence (Keysight signal-analysis ML, Rohde & \
                Schwarz spectrum monitoring AI, NI RFIC analyser ML, Ettus USRP-based \
                learned pipelines; Restuccia / Sankhe / Wang RF fingerprinting)",
            year: 2023,
            venue: "Vendor documentation + RF fingerprinting literature (rejection- \
                shell reference; vendor scores and learned classifiers lack public \
                deterministic feature-extraction law)",
        },
    ]
}

/// Build the T.12.m RF / communications `DedupCourtDelta`.
/// The delta names SIX new canonicals at 6303..=6308; the CFO
/// and EVM ideas that once shadowed 6301 / 6302 collapsed onto
/// SEED 53 and SEED 54 under the SEED-walk-first discipline
/// and appear as `ExistingCanonicalAuthorityResolution` records
/// in the body, NOT in `new_canonical_records`.
fn build_rf_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_m_rf_delta",
        vec![
            DetectorCanonicalId(CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CIR_DRIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(IQ_IMBALANCE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PHASE_NOISE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID),
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

/// Build the T.12.m RF / communications `CorpusAmendmentProposal`.
/// Two builds against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_m_rf_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_m_rf_first_proposal",
        "T.12.m files the RF / Communications amendment proposal. Adds SIX \
         genuinely new canonical RF primitives (constellation spread, channel \
         impulse response drift, IQ imbalance, phase-noise, symbol-timing offset \
         residual, cyclostationary feature shift) at reserved canonical ids \
         6303..=6308 with declared signal representation + sampling law + unit \
         law + carrier / channel assumption + synchronization assumption + window \
         / transform law + decision functional + confuser profile + numeric mode \
         contracts. Records SIX ExistingCanonicalAuthorityResolution decisions \
         keeping FFT band-energy (SEED 12), Residual envelope exit (22), Spectral \
         entropy (38), Autocorrelation break (40), Carrier-frequency-offset \
         residual (53; Morelli & Mengali 1999), and Error Vector Magnitude \
         anomaly (54; Shafik / Rahman / Islam 2006) canonical under \
         RfCommunications. Reserved ids 6301 and 6302 are deliberately unused — \
         the CFO and EVM ideas that once shadowed them collapsed onto SEED 53 \
         and SEED 54 respectively under the SEED-walk-first discipline. Records \
         TWO DomainTransferOf decisions: SEED 12 as shared spectral RF ancestor; \
         SEED 22 as shared envelope-boundary RF ancestor. Records FOUR \
         ParameterizationOf decisions (panel-candidate canonicals that collapsed \
         on closer inspection): spectral mask violation (6309) is \
         ParameterizationOf(FFT band-energy, SEED 12); SNR drop (6310) is \
         ParameterizationOf(FFT band-energy, SEED 12); burst preamble miss \
         (6311) is ParameterizationOf(Autocorrelation break, SEED 40); \
         frame-error burst (6312) is ParameterizationOf(Error burst, SEED 41). \
         Rejects TWO RF records as RejectedNotDeterministic (seventh T.12.x with \
         two rejections, following T.12.g / h / i / j / k / l): learned RF \
         fingerprinting classifier (6313; Restuccia 2019 DeepRadioID, Sankhe \
         2019 ORACLE, Wang 2022) and black-box modulation classifier / \
         proprietary spectrum anomaly score (6314; Keysight signal-analysis ML, \
         Rohde & Schwarz spectrum monitoring AI, NI RFIC analyser ML, Ettus \
         USRP-based learned pipelines). Panel-locked non-claim: T.12.m admits \
         deterministic RF / communications signal witnesses, not emitter \
         attribution, transmitter identification, geolocation, spectrum- \
         enforcement authority, military classification, or communications- \
         intelligence conclusions. Every CanonicalAddition / \
         ExistingCanonicalAuthorityResolution reason text declares the full \
         contract AND ends with the panel-locked non-claim 'RF signal witness, \
         not emitter attribution or spectrum enforcement' (pinned by \
         t12_m_rejects_emitter_identification_claim_language, \
         t12_m_rejects_geolocation_or_attribution_claim_language, and \
         t12_m_rejects_spectrum_enforcement_claim_language scanners). Does NOT \
         mutate SEED (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::RfCommunications,
        build_rf_expansion_batch(),
        build_rf_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_m_rf",
    )
}
