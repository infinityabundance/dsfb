//! T.12.p — Information Theory catch-up: the sixteenth real
//! literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.p files the Information Theory catch-up amendment
//! > proposal. It admits only deterministic information-theoretic
//! > witnesses: entropy, conditional entropy, mutual information,
//! > cross-entropy / negative-log-likelihood, and minimum
//! > description length / coding-length residuals whose
//! > estimator, binning, smoothing, sample-support, joint-
//! > distribution contract, log base, empty-bin law, and numeric
//! > mode are declared; resolves SEED collisions with KL
//! > divergence, Jensen-Shannon divergence, and Spectral entropy;
//! > classifies variants as parameterizations or domain transfers;
//! > rejects learned mutual-information estimators and black-box
//! > information-theoretic anomaly scores without declared
//! > deterministic-binning / kernel / partition / formula
//! > contract; and preserves the frozen T.10 corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"An information-
//! theoretic witness is admissible only when the estimator,
//! binning or kernel, smoothing, sample-support, joint-
//! distribution contract (where applicable), log base, empty-bin
//! law, and numeric mode are declared."*
//!
//! **Panel-locked non-claim (MUST appear verbatim in receipt /
//! README / paper)**:
//!
//! > T.12.p admits deterministic information-theoretic witnesses:
//! > entropy, divergence, mutual-information, coding-length,
//! > compression, surprise, and dependence-structure evidence
//! > with declared estimator, binning, smoothing, sample-support,
//! > and numeric laws. It does not admit semantic meaning, causal
//! > information flow certainty, privacy leakage certainty,
//! > cryptographic security claims, or learned representation
//! > claims.
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.p's design began with a grep of [`crate::seed::SEED`]
//! for every information-theoretic candidate. The walk found
//! **three** T.12.p-relevant primitives already canonical:
//!
//! * **Kullback-Leibler divergence** at SEED id 9 — the
//!   foundational divergence functional from Kullback-Leibler
//!   1951. Every "relative entropy" / "I-divergence" claim
//!   collapses here.
//! * **Jensen-Shannon divergence** at SEED id 32 — the symmetric
//!   bounded JS variant from Lin 1991. Every JSD / Jensen
//!   difference claim collapses here.
//! * **Spectral entropy** at SEED id 38 — Shannon entropy on the
//!   power spectrum per Inouye 1991. Every "spectral Shannon
//!   entropy" / "power-spectrum entropy" claim collapses here.
//!
//! All three become `ExistingCanonicalAuthorityResolution`
//! records under the `InformationTheory` source class.
//! **Panel-locked success-shape** (mirroring T.12.k / T.12.l /
//! T.12.m / T.12.n / T.12.o): the campaign's strength comes from
//! cross-class dedup discipline (3 authority resolutions over
//! the KL / JS / Spectral-entropy SEED family that information-
//! theoretic witnesses heavily reuse), not detector count.
//!
//! Five genuinely new canonicals at reserved ids 6601..=6605
//! survived the SEED-walk as structurally distinct information-
//! theoretic decision functionals. Each declares an explicit
//! contract over estimator (plug-in / Miller-Madow / kernel /
//! James-Stein / discrete-binned), binning or partition law,
//! smoothing (Laplace / Krichevsky-Trofimov / none), sample-
//! support bound, log base, empty-bin law, joint-distribution
//! contract (where applicable), and numeric mode; each is an
//! INFORMATION-THEORETIC decision functional on an empirical
//! probability mass / density estimate (not a raw-stream
//! decision):
//!
//! * **Shannon entropy shift witness** (6601; Shannon 1948 A
//!   Mathematical Theory of Communication). Declares base
//!   (typically log2 / bits or ln / nats), binning or partition
//!   law (equal-width / equal-frequency / Freedman-Diaconis /
//!   declared partition function), empty-bin law (skip / Laplace
//!   smoothing alpha / Krichevsky-Trofimov 1/2), smoothing rule,
//!   and sample-support bound (minimum samples per bin or per
//!   partition). Per-window entropy estimate residual vs baseline
//!   reference entropy.
//! * **Conditional entropy shift witness** (6602; Cover-Thomas
//!   2006 Elements of Information Theory chapter 2). Declares
//!   joint-distribution contract over (X, Y), binning law for
//!   both marginals AND the joint, empty-bin law, smoothing
//!   rule, sample-support bound, and log base. Per-window H(Y|X)
//!   = H(X,Y) - H(X) estimate residual vs baseline reference.
//! * **Mutual information break witness** (6603; Cover-Thomas
//!   2006 chapter 2). Declares joint-distribution contract over
//!   (X, Y), binning OR kernel-density-estimator law for both
//!   marginals AND the joint, empty-bin law, smoothing rule,
//!   sample-support bound (with declared bias-correction rule:
//!   Miller-Madow / James-Stein / none), and log base. Per-window
//!   I(X; Y) = H(X) + H(Y) - H(X, Y) estimate break vs baseline.
//!   Structurally distinct from SEED 9 KL because MI is a
//!   functional on the JOINT vs PRODUCT-OF-MARGINALS, whereas
//!   KL is a divergence between two declared distributions.
//! * **Cross-entropy / negative-log-likelihood residual witness**
//!   (6604; Shannon 1948 / Cover-Thomas 2006). Declares FIXED
//!   MODEL distribution q (parameter-pinned; no learned
//!   parameters at decision time), empirical sample distribution
//!   p, log base, smoothing (epsilon for log(0)), and sample-
//!   support bound. Per-window H(p, q) = -sum_i p_i log q_i
//!   residual vs baseline. Sketch-state decision only; the court
//!   does NOT admit deterministic likelihood-truth claims.
//! * **Minimum description length / coding-length residual
//!   witness** (6605; Rissanen 1978 Modeling by Shortest Data
//!   Description; Rissanen 1986 Stochastic Complexity and
//!   Modeling). Declares model class (fixed prefix code / fixed
//!   universal code / declared two-part code with parameter-cost
//!   law), code-length functional L(D | M), L(M) decomposition,
//!   sample-support bound, and numeric mode. Per-window total code
//!   length residual L(D | M) + L(M) vs baseline. The court does
//!   NOT admit model-selection truth or causal-explanation
//!   claims.
//!
//! ## Authority-resolution records (3)
//!
//! Each of SEED 9, 32, 38 stays canonical with declared
//! information-theoretic contract:
//!
//! * SEED 9 KL divergence — declared reference distribution q +
//!   empirical distribution p + log base + epsilon for log(0) +
//!   empty-bin law + sample-support bound. Streaming descendants
//!   point here.
//! * SEED 32 Jensen-Shannon divergence — declared symmetric
//!   mixture M = 0.5(P + Q) + bounded JS = 0.5 KL(P||M) + 0.5
//!   KL(Q||M) + log base + sample-support bound.
//! * SEED 38 Spectral entropy — Shannon entropy applied to the
//!   normalised power spectrum + declared FFT window size +
//!   spectral-bin contract + empty-bin law + log base.
//!
//! ## ParameterizationOf records (4)
//!
//! Panel-candidate primitives that collapsed on closer
//! inspection because they are deterministic re-expressions of
//! the 6601 / 6603 / 6605 canonicals:
//!
//! * **Normalized mutual information** (6606) →
//!   `ParameterizationOf(Mutual information, 6603)` with declared
//!   normalisation function (arithmetic mean / geometric mean /
//!   max / joint-entropy). NMI is a scaled MI; the underlying
//!   decision functional is the same.
//! * **Transfer entropy proxy** (6607; Schreiber 2000 Measuring
//!   Information Transfer) → `ParameterizationOf(Mutual
//!   information, 6603)` with declared lagged-joint contract
//!   (TE_{X→Y} = I(Y_{t+1}; X_t^{(k)} | Y_t^{(l)})). ADMITTED
//!   ONLY AS A DETERMINISTIC NON-CAUSAL WITNESS. The court does
//!   NOT admit causal-information-flow claims; transfer-entropy
//!   directionality is a deterministic descriptive statistic on
//!   the lagged joint, not evidence of causal structure.
//! * **Rényi entropy / Tsallis entropy** (6608; Rényi 1961 / Tsallis
//!   1988) → `ParameterizationOf(Shannon entropy, 6601)` with
//!   declared order-alpha parameter law AND declared base AND
//!   declared limit-recovery (Rényi alpha=1 = Shannon; Tsallis
//!   q=1 = Shannon). Admitted only with parameter law explicit.
//! * **Compression-ratio anomaly** (6609; Ziv-Lempel 1977 / Ziv-
//!   Lempel 1978 / Welch 1984) → `ParameterizationOf(Minimum
//!   description length, 6605)` with declared compression
//!   algorithm (LZ77 / LZ78 / LZW / gzip / bzip2 / xz with
//!   declared parameters) + declared compression-ratio decision
//!   functional. The court does NOT admit compression as a
//!   surrogate for true description length; compression-ratio is
//!   a sketch-state proxy with declared algorithm.
//!
//! ## RejectedNotDeterministic records (2)
//!
//! Tenth T.12.x with two RejectedNotDeterministic records in one
//! commit, following T.12.g / h / i / j / k / l / m / n / o.
//!
//! * **Learned mutual-information estimator** (6610; MINE
//!   Belghazi et al. 2018 Mutual Information Neural Estimation /
//!   InfoMax / variational MI bounds / neural KL estimators).
//!   These pipelines estimate MI with a trained neural network
//!   without a declared deterministic binning / kernel /
//!   partition law. Rejected unless reduced to a
//!   `Deterministic_MI_Estimator_Proxy` with deterministic
//!   feature-extraction law / declared formula / declared
//!   training-data anchor / declared binning or kernel / declared
//!   tie-break / declared numeric mode / no learned opaque
//!   embedding.
//! * **Black-box information-theoretic anomaly score** (6611;
//!   vendor pipelines: AWS Macie information-leakage scoring,
//!   IBM Guardium DAM information-theoretic anomaly heuristics,
//!   Microsoft Purview information-leakage classifier,
//!   Symantec / Broadcom DLP entropy-based anomaly score, Cisco
//!   Talos information-theoretic threat scoring) without
//!   declared base / log / smoothing / sample-correction law.
//!   Rejected unless reduced to a `Deterministic_IT_Score_Proxy`
//!   with declared formula + binning + smoothing + sample-
//!   support + log base + numeric mode. The court does NOT issue
//!   verdicts from black-box vendor IT pipelines; the rejection-
//!   shell describes what is NOT admitted.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories:
//!
//! * `CanonicalAddition` ×5 (6601..=6605).
//! * `ExistingCanonicalAuthorityResolution` ×3 — SEED 9, 32, 38.
//! * `DomainTransferOf` ×2 — SEED 9 (divergence ancestor) +
//!   SEED 38 (entropy-on-distribution ancestor).
//! * `ParameterizationOf` ×4.
//! * `RejectedNotDeterministic` ×2.
//!
//! Total: 5 + 3 + 2 + 4 + 2 = **16 dedup-court records**.
//!
//! ## Causal-flow / privacy / security / learned-embedding
//! discipline (panel-locked, MOST IMPORTANT)
//!
//! Every CanonicalAddition AND
//! ExistingCanonicalAuthorityResolution reason text MUST
//! describe its record as a "deterministic information-theoretic
//! witness" with declared estimator / binning / smoothing /
//! support contracts, NEVER as a causal-information-flow /
//! privacy-leakage / cryptographic-security / learned-
//! representation claim. The dedicated load-bearing negatives
//! scan every such reason for forbidden terms across six
//! parametric scanners:
//!
//! - missing estimator / binning contract;
//! - missing entropy base / smoothing / empty-bin law;
//! - missing joint-distribution contract on MI / conditional
//!   entropy;
//! - causal-information-flow claim language (causal flow,
//!   causation, granger-style causal, causal influence,
//!   intervention truth);
//! - privacy / security claim language (privacy-leakage
//!   certainty, cryptographic security, side-channel-secure,
//!   information-theoretically secure encryption verdict);
//! - learned-embedding information-score claim language (neural
//!   MI estimator certainty, learned representation MI truth,
//!   embedding-based entropy verdict).
//!
//! Forbidden terms appear ONLY in `RejectedNotDeterministic`
//! reason text (where they describe what is NOT admitted).
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11 / S1.3 / T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13 / 89 / 1917 byte-stable.
//! * **NEW**: a non-trivial T.12.p information-theory
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
// Reserved id constants (panel-locked, 6601..=6611 used;
// 6612..=6699 reserved for future Information Theory proposals)
// ---------------------------------------------------------------

/// Reserved canonical id for Shannon entropy shift witness
/// (Shannon 1948). Declared base + binning / partition law +
/// empty-bin law + smoothing + sample-support bound.
pub const SHANNON_ENTROPY_RESERVED_CANONICAL_ID: u32 = 6601;

/// Reserved canonical id for Conditional entropy shift witness
/// (Cover-Thomas 2006). Declared joint-distribution contract
/// over (X, Y) + binning + empty-bin law + smoothing + sample-
/// support bound + log base.
pub const CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID: u32 = 6602;

/// Reserved canonical id for Mutual information break witness
/// (Cover-Thomas 2006). Declared joint-distribution contract +
/// binning or kernel-density-estimator + empty-bin law +
/// smoothing + bias-correction rule (Miller-Madow / James-Stein
/// / none) + log base. Structurally distinct from SEED 9 KL
/// because MI is a functional on the JOINT vs PRODUCT-OF-
/// MARGINALS whereas KL is a divergence between two declared
/// distributions.
pub const MUTUAL_INFORMATION_RESERVED_CANONICAL_ID: u32 = 6603;

/// Reserved canonical id for Cross-entropy / negative-log-
/// likelihood residual witness (Shannon 1948 / Cover-Thomas
/// 2006). Declared FIXED MODEL distribution q (parameter-pinned;
/// no learned parameters at decision time) + empirical sample
/// distribution p + log base + smoothing (epsilon for log(0)) +
/// sample-support bound.
pub const CROSS_ENTROPY_RESERVED_CANONICAL_ID: u32 = 6604;

/// Reserved canonical id for Minimum description length /
/// coding-length residual witness (Rissanen 1978 / Rissanen
/// 1986). Declared model class + code-length functional
/// L(D | M) + L(M) decomposition + sample-support bound +
/// numeric mode.
pub const MDL_RESERVED_CANONICAL_ID: u32 = 6605;

/// Reserved id for Normalized mutual information.
/// `ParameterizationOf(Mutual information, 6603)`.
pub const NORMALIZED_MI_RESERVED_PRIMITIVE_ID: u32 = 6606;

/// Reserved id for Transfer entropy proxy (Schreiber 2000;
/// admitted ONLY AS A DETERMINISTIC NON-CAUSAL WITNESS).
/// `ParameterizationOf(Mutual information, 6603)`.
pub const TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID: u32 = 6607;

/// Reserved id for Rényi entropy / Tsallis entropy (Rényi 1961
/// / Tsallis 1988). `ParameterizationOf(Shannon entropy, 6601)`.
pub const RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID: u32 = 6608;

/// Reserved id for Compression-ratio anomaly (Ziv-Lempel 1977 /
/// Ziv-Lempel 1978 / Welch 1984).
/// `ParameterizationOf(Minimum description length, 6605)`.
pub const COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID: u32 = 6609;

/// Reserved id for Learned mutual-information estimator (MINE
/// Belghazi et al. 2018 / variational MI bounds / neural KL
/// estimators). `RejectedNotDeterministic`.
pub const LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID: u32 = 6610;

/// Reserved id for Black-box information-theoretic anomaly score
/// without declared base / log / smoothing / sample-correction
/// law. `RejectedNotDeterministic`.
pub const BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID: u32 = 6611;

// Existing SEED canonical ids referenced by T.12.p.

/// Kullback-Leibler divergence — SEED canonical id 9. Shared
/// information-theoretic divergence ancestor; every relative-
/// entropy / I-divergence claim collapses here.
pub const KL_SEED_ID: u32 = 9;

/// Jensen-Shannon divergence — SEED canonical id 32. Symmetric
/// bounded JS variant; every JSD / Jensen difference claim
/// collapses here.
pub const JS_SEED_ID: u32 = 32;

/// Spectral entropy — SEED canonical id 38. Shannon entropy on
/// the normalised power spectrum; every spectral Shannon entropy
/// / power-spectrum entropy claim collapses here.
pub const SPECTRAL_ENTROPY_SEED_ID: u32 = 38;

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
// Builders for the information-theory expansion batch
// ---------------------------------------------------------------

/// Build the information-theory `CorpusExpansionBatch` body.
fn build_information_theory_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_p_information_theory_first_proposal",
        SourceClass::InformationTheory,
        information_theory_proposed_primitives(),
        information_theory_proposed_aliases(),
        information_theory_proposed_dedup_records(),
        information_theory_proposed_genealogy_edges(),
        information_theory_proposed_source_refs(),
    )
}

/// Eleven proposed primitives: 5 canonical (the information-
/// theoretic primitives that survived SEED-walk as structurally
/// distinct functionals) + 4 parameterization shells + 2
/// rejection shells. The "tight canonical set, heavy contract
/// discipline around estimator / binning / smoothing / joint-
/// distribution contract, clear rejection of learned MI
/// estimators AND black-box vendor IT scores without contract
/// declaration" shape applies the panel-locked T.12.k / T.12.l /
/// T.12.m / T.12.n / T.12.o success posture to information
/// theory.
fn information_theory_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            display_name: "Shannon entropy shift witness",
            motivation: "Shannon entropy shift information-theoretic witness \
                 (Shannon 1948 A Mathematical Theory of Communication). \
                 Required contract: log base (typically log2 / bits or ln / \
                 nats; declared per record), binning or partition law (equal-\
                 width / equal-frequency / Freedman-Diaconis / declared \
                 partition function), empty-bin law (skip / Laplace smoothing \
                 alpha / Krichevsky-Trofimov 1/2), smoothing rule, sample-\
                 support bound (minimum samples per bin or per partition), \
                 estimator (plug-in / Miller-Madow / James-Stein / declared), \
                 residual definition (per-window entropy estimate vs baseline \
                 reference entropy), decision functional (per-window entropy \
                 shift exceeds threshold), confuser profile (small-sample \
                 estimation bias, partition mismatch, sparse-bin artefacts, \
                 stationarity violation), numeric mode. Deterministic \
                 information-theoretic witness; the court does NOT admit \
                 semantic-meaning or causal-information-flow claims.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID),
            display_name: "Conditional entropy shift witness",
            motivation: "Conditional entropy shift information-theoretic witness \
                 (Cover-Thomas 2006 Elements of Information Theory chapter 2). \
                 Required contract: joint-distribution contract over (X, Y) \
                 (declared joint sampling law + joint binning or partition \
                 over the product space), binning law for both marginals AND \
                 the joint (consistent partition function), empty-bin law for \
                 the joint cells (skip / Laplace / Krichevsky-Trofimov), \
                 smoothing rule, sample-support bound (minimum joint samples \
                 per joint cell), estimator (plug-in / Miller-Madow / James-\
                 Stein), log base, residual definition (per-window \
                 H(Y|X) = H(X,Y) - H(X) estimate vs baseline reference), \
                 decision functional, confuser profile (joint-sparsity bias, \
                 marginal-vs-joint binning mismatch, sample-size shift), \
                 numeric mode. Deterministic information-theoretic witness; \
                 the court does NOT admit causal-information-flow certainty.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            display_name: "Mutual information break witness",
            motivation: "Mutual information break information-theoretic witness \
                 (Cover-Thomas 2006 chapter 2). Required contract: joint-\
                 distribution contract over (X, Y) (declared joint sampling \
                 law + joint binning or kernel-density-estimator over the \
                 product space), binning law for both marginals AND the joint, \
                 empty-bin law (skip / Laplace / Krichevsky-Trofimov), \
                 smoothing rule, bias-correction rule (Miller-Madow correction \
                 plus_or_minus (K - 1) / (2 N ln 2) / James-Stein shrinkage / \
                 none; declared), sample-support bound, log base, residual \
                 definition (per-window I(X; Y) = H(X) + H(Y) - H(X, Y) \
                 estimate break vs baseline), decision functional, confuser \
                 profile (joint-sparsity bias, finite-sample positive bias \
                 without correction, partition mismatch, redundant features), \
                 numeric mode. Structurally distinct from SEED 9 KL divergence \
                 because MI is a functional on the JOINT distribution vs the \
                 PRODUCT OF MARGINALS, whereas KL is a divergence between two \
                 declared distributions. Deterministic information-theoretic \
                 witness; the court does NOT admit causal-information-flow \
                 verdicts; mutual information is symmetric and non-directional \
                 by construction.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CROSS_ENTROPY_RESERVED_CANONICAL_ID),
            display_name: "Cross-entropy / negative-log-likelihood residual witness",
            motivation: "Cross-entropy / negative-log-likelihood residual \
                 information-theoretic witness (Shannon 1948 / Cover-Thomas \
                 2006). Required contract: FIXED MODEL distribution q (\
                 parameter-pinned; no learned parameters at decision time; \
                 the model law and parameter values are declared in the \
                 record and frozen across the comparison window) + empirical \
                 sample distribution p (declared per-window estimate) + log \
                 base + smoothing (epsilon for log(0) values; declared \
                 epsilon parameter) + empty-bin law + sample-support bound + \
                 residual definition (per-window H(p, q) = -sum_i p_i log q_i \
                 residual vs baseline) + decision functional (per-window \
                 cross-entropy shift exceeds threshold) + confuser profile \
                 (model misspecification, support mismatch, epsilon-sensitive \
                 zero-probability cells, sample-size shift) + numeric mode. \
                 Deterministic information-theoretic witness; the court does \
                 NOT admit deterministic likelihood-truth claims or learned-\
                 representation claims.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MDL_RESERVED_CANONICAL_ID),
            display_name: "Minimum description length / coding-length residual witness",
            motivation: "Minimum description length / coding-length residual \
                 information-theoretic witness (Rissanen 1978 Modeling by \
                 Shortest Data Description / Rissanen 1986 Stochastic \
                 Complexity and Modeling). Required contract: model class M \
                 (declared as fixed prefix code / fixed universal code / two-\
                 part code with declared parameter-cost law) + code-length \
                 functional L(D | M) (declared per-sample code-length formula) \
                 + L(M) parameter-encoding cost (declared two-part code; the \
                 model-cost is not silently dropped) + sample-support bound + \
                 residual definition (per-window total code length L(D | M) + \
                 L(M) residual vs baseline) + decision functional (per-window \
                 total code length exceeds threshold) + confuser profile \
                 (model-class mismatch, parameter-cost-law sensitivity, \
                 sample-size shift) + numeric mode. Deterministic information-\
                 theoretic witness; the court does NOT admit model-selection \
                 truth or causal-explanation claims; MDL is a deterministic \
                 description-length residual, not a causal model verdict.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(NORMALIZED_MI_RESERVED_PRIMITIVE_ID),
            display_name: "Normalized mutual information - parameterization shell",
            motivation: "Normalized mutual information is a ParameterizationOf \
                 the Mutual information canonical (6603) with declared \
                 normalisation function (arithmetic mean of marginal \
                 entropies, geometric mean of marginal entropies, max of \
                 marginal entropies, or joint entropy; declared per record). \
                 The underlying decision functional is the same MI computation \
                 with a scaling transform; the court declines to admit \
                 normalised MI as a separate canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID),
            display_name: "Transfer entropy proxy - parameterization shell (non-causal)",
            motivation: "Transfer entropy proxy (Schreiber 2000 Measuring \
                 Information Transfer) is a ParameterizationOf the Mutual \
                 information canonical (6603) with declared lagged-joint \
                 contract: TE_{X->Y} = I(Y_{t+1}; X_t^{(k)} | Y_t^{(l)}) over \
                 declared embedding orders k and l, declared lag, and declared \
                 joint binning over the lagged product space. ADMITTED ONLY \
                 AS A DETERMINISTIC NON-CAUSAL WITNESS: transfer-entropy \
                 directionality is a deterministic descriptive statistic on \
                 the lagged joint distribution; the court does NOT admit \
                 transfer entropy as evidence of causal structure, intervention \
                 truth, or causal information flow. The court declines to \
                 admit transfer entropy as a separate canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID),
            display_name: "Rényi / Tsallis entropy - parameterization shell",
            motivation: "Rényi entropy (Rényi 1961 On Measures of Entropy and \
                 Information) and Tsallis entropy (Tsallis 1988 Possible \
                 Generalization of Boltzmann-Gibbs Statistics) together form \
                 a ParameterizationOf the Shannon entropy canonical (6601) \
                 with declared order-alpha parameter law AND declared base \
                 AND declared limit-recovery (Rényi alpha=1 reduces to \
                 Shannon entropy; Tsallis q=1 reduces to Shannon entropy). \
                 Admitted only with parameter law explicit; the court \
                 declines to admit Rényi or Tsallis entropy as a separate \
                 canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID),
            display_name: "Compression-ratio anomaly - parameterization shell",
            motivation: "Compression-ratio anomaly (Ziv-Lempel 1977 / Ziv-\
                 Lempel 1978 / Welch 1984 LZW) is a ParameterizationOf the \
                 Minimum description length canonical (6605) with declared \
                 compression algorithm (LZ77 / LZ78 / LZW / gzip / bzip2 / \
                 xz with declared compression-level and dictionary-size \
                 parameters) + declared compression-ratio decision functional \
                 (per-window compressed-byte-count vs raw-byte-count ratio \
                 residual). Compression-ratio is a sketch-state proxy for \
                 MDL with a declared algorithm; the court declines to admit \
                 compression-ratio anomaly as a separate canonical primitive \
                 AND does NOT admit compression as a surrogate for true \
                 description length.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID),
            display_name: "Learned mutual-information estimator - rejected shell",
            motivation: "Learned mutual-information estimators (MINE Belghazi \
                 et al. 2018 Mutual Information Neural Estimation; InfoMax / \
                 variational MI bounds; neural KL estimators; InfoVAE; CPC \
                 contrastive predictive coding MI lower bounds) estimate MI \
                 with a trained neural network without a declared \
                 deterministic binning / kernel / partition law. The court \
                 does NOT admit these as canonical witnesses because the \
                 decision functional depends on opaque learned weights. \
                 Admission requires a future T.12.x to admit a \
                 Deterministic_MI_Estimator_Proxy with deterministic feature-\
                 extraction law + declared formula + declared training-data \
                 anchor (pinned dataset record-hash) + declared binning OR \
                 kernel + declared tie-break law + declared numeric mode + \
                 no learned opaque embedding, all brutally explicit.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID),
            display_name: "Black-box information-theoretic anomaly score - rejected shell",
            motivation: "Black-box information-theoretic anomaly scores from \
                 vendor pipelines (AWS Macie information-leakage scoring; IBM \
                 Guardium DAM information-theoretic anomaly heuristics; \
                 Microsoft Purview information-leakage classifier; Symantec / \
                 Broadcom DLP entropy-based anomaly score; Cisco Talos \
                 information-theoretic threat scoring) expose anomaly verdicts \
                 without declaring the underlying log base, smoothing rule, \
                 empty-bin law, partition function, or sample-correction law. \
                 The court does NOT admit these as canonical witnesses because \
                 the decision functional cannot be replayed without the \
                 contract. A future T.12.x may admit a \
                 Deterministic_IT_Score_Proxy with declared formula + binning \
                 / partition + smoothing + sample-support bound + log base + \
                 numeric mode (either via vendor publication or user \
                 deployment-time configuration pinned in a fixed receipt).",
        },
    ]
}

/// Zero alias claims (T.12.p routes everything through dedup
/// records and existing-canonical authority resolutions).
fn information_theory_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Sixteen dedup-court decisions on the information-theory batch.
fn information_theory_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 5 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            reason: "Shannon entropy shift witness (Shannon 1948): declared \
                 log base (log2 / bits or ln / nats) + binning or partition \
                 law (equal-width / equal-frequency / Freedman-Diaconis / \
                 declared partition function) + empty-bin law (skip / \
                 Laplace smoothing alpha / Krichevsky-Trofimov 1/2) + \
                 smoothing rule + sample-support bound + estimator (plug-in \
                 / Miller-Madow / James-Stein / declared) + residual \
                 definition + decision functional + confuser profile + \
                 numeric mode. Deterministic information-theoretic witness; \
                 the court does NOT admit semantic meaning, causal \
                 information flow certainty, or learned representation \
                 claims.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID),
            reason: "Conditional entropy shift witness (Cover-Thomas 2006 \
                 chapter 2): declared joint-distribution contract over \
                 (X, Y) + joint binning or partition over the product space \
                 + binning law for both marginals AND the joint + empty-bin \
                 law for joint cells + smoothing rule + sample-support \
                 bound + estimator + log base + residual definition (per-\
                 window H(Y|X) = H(X,Y) - H(X) vs baseline) + decision \
                 functional + confuser profile + numeric mode. Deterministic \
                 information-theoretic witness; the court does NOT admit \
                 causal information flow certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            reason: "Mutual information break witness (Cover-Thomas 2006 \
                 chapter 2): declared joint-distribution contract over \
                 (X, Y) + binning OR kernel-density-estimator law for both \
                 marginals AND the joint + empty-bin law + smoothing rule + \
                 bias-correction rule (Miller-Madow / James-Stein / none; \
                 declared) + sample-support bound + log base + residual \
                 definition (per-window I(X; Y) = H(X) + H(Y) - H(X, Y) \
                 estimate vs baseline) + decision functional + confuser \
                 profile + numeric mode. Structurally distinct from SEED 9 \
                 KL divergence: MI is a functional on the JOINT distribution \
                 vs the PRODUCT OF MARGINALS, whereas KL is a divergence \
                 between two declared distributions. Deterministic \
                 information-theoretic witness; the court does NOT admit \
                 causal information flow certainty; mutual information is \
                 symmetric and non-directional by construction.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CROSS_ENTROPY_RESERVED_CANONICAL_ID),
            reason: "Cross-entropy / negative-log-likelihood residual witness \
                 (Shannon 1948 / Cover-Thomas 2006): declared FIXED MODEL \
                 distribution q (parameter-pinned; the model law and \
                 parameter values are declared and frozen across the \
                 comparison window; no learned parameters at decision time) \
                 + empirical sample distribution p (declared estimator: \
                 plug-in / Miller-Madow / declared) + log base + smoothing \
                 (epsilon for log(0)) + empty-bin law + sample-support \
                 bound + residual definition (per-window H(p, q) = -sum_i \
                 p_i log q_i vs baseline) + decision functional + confuser \
                 profile + numeric mode. Deterministic information-\
                 theoretic witness; the court does NOT admit deterministic \
                 likelihood-truth claims or learned representation claims.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MDL_RESERVED_CANONICAL_ID),
            reason: "Minimum description length / coding-length residual \
                 witness (Rissanen 1978 / Rissanen 1986): declared model \
                 class (fixed prefix code / fixed universal code / two-part \
                 code with declared parameter-cost law) + code-length \
                 functional L(D | M) (declared per-sample code-length \
                 formula) + L(M) parameter-encoding cost (declared two-part \
                 code; model-cost is not silently dropped) + sample-support \
                 bound + residual definition (per-window total code length \
                 L(D | M) + L(M) vs baseline) + decision functional + \
                 confuser profile + numeric mode. Deterministic information-\
                 theoretic witness; the court does NOT admit model-selection \
                 truth or causal-explanation claims; MDL is a deterministic \
                 description-length residual, not a causal model verdict.",
        },
        // -- 3 ExistingCanonicalAuthorityResolution records ---
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(KL_SEED_ID),
            reason: "Kullback-Leibler divergence stays canonical at SEED id \
                 9 under InformationTheory. Declared reference distribution \
                 q + empirical sample distribution p + log base + epsilon \
                 for log(0) + empty-bin law + sample-support bound + \
                 estimator + numeric mode. KL = sum_i p_i log(p_i / q_i); \
                 the underlying decision functional remains the asymmetric \
                 information-theoretic divergence between two declared \
                 distributions. Streaming descendants and every relative-\
                 entropy / I-divergence claim collapse here; no duplicate \
                 admitted. Deterministic information-theoretic witness; the \
                 court does NOT admit causal information flow certainty or \
                 privacy / cryptographic security claims.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(JS_SEED_ID),
            reason: "Jensen-Shannon divergence stays canonical at SEED id \
                 32 under InformationTheory. Declared symmetric mixture M = \
                 0.5 (P + Q) + bounded JS = 0.5 KL(P||M) + 0.5 KL(Q||M) + \
                 log base + empty-bin law + sample-support bound + \
                 estimator + numeric mode. The underlying decision \
                 functional remains the symmetric bounded JS divergence. \
                 Every JSD / Jensen difference claim collapses here; no \
                 duplicate admitted. Deterministic information-theoretic \
                 witness; the court does NOT admit causal information flow \
                 certainty.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            reason: "Spectral entropy stays canonical at SEED id 38 under \
                 InformationTheory. Shannon entropy applied to the \
                 normalised power spectrum (Inouye 1991): declared FFT \
                 window size + spectral-bin contract + empty-bin law + log \
                 base + estimator + numeric mode. The underlying decision \
                 functional remains the Shannon-entropy-on-power-spectrum \
                 irregularity metric. Every spectral Shannon entropy / \
                 power-spectrum entropy claim collapses here; no duplicate \
                 admitted. Deterministic information-theoretic witness; the \
                 court does NOT admit semantic meaning or causal information \
                 flow certainty.",
        },
        // -- 2 DomainTransferOf records ----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(KL_SEED_ID),
            reason: "Kullback-Leibler divergence (SEED id 9) is the shared \
                 information-theoretic divergence ancestor for the \
                 InformationTheory source class. Cross-entropy (6604) and \
                 Jensen-Shannon divergence (SEED 32) are descendants. The \
                 court records the domain transfer without re-canonicalising \
                 KL divergence.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            reason: "Spectral entropy (SEED id 38) is the shared Shannon-\
                 entropy-on-distribution ancestor for the InformationTheory \
                 source class. Shannon entropy shift (6601) is the abstract \
                 parent admitted as a new canonical; spectral entropy is the \
                 power-spectrum-domain variant already canonical. The court \
                 records the domain transfer without re-canonicalising \
                 spectral entropy.",
        },
        // -- 4 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(NORMALIZED_MI_RESERVED_PRIMITIVE_ID),
            reason: "Normalized mutual information is ParameterizationOf \
                 (Mutual information, 6603) with declared normalisation \
                 function (arithmetic mean of marginal entropies, geometric \
                 mean of marginal entropies, max of marginal entropies, or \
                 joint entropy; declared per record). The underlying \
                 decision functional is the same MI computation with a \
                 scaling transform. The court declines to admit normalised \
                 MI as a separate canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID),
            reason: "Transfer entropy proxy (Schreiber 2000) is \
                 ParameterizationOf(Mutual information, 6603) with declared \
                 lagged-joint contract: TE_{X->Y} = I(Y_{t+1}; X_t^{(k)} | \
                 Y_t^{(l)}) over declared embedding orders k and l, \
                 declared lag, and declared joint binning over the lagged \
                 product space. ADMITTED ONLY AS A DETERMINISTIC NON-CAUSAL \
                 WITNESS: transfer-entropy directionality is a deterministic \
                 descriptive statistic on the lagged joint distribution; \
                 the court does NOT admit transfer entropy as evidence of \
                 causal structure, intervention truth, or causal information \
                 flow. The court declines to admit transfer entropy as a \
                 separate canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID),
            reason: "Rényi entropy (Rényi 1961) and Tsallis entropy (Tsallis \
                 1988) are ParameterizationOf(Shannon entropy, 6601) with \
                 declared order-alpha parameter law AND declared base AND \
                 declared limit-recovery (Rényi alpha=1 reduces to Shannon; \
                 Tsallis q=1 reduces to Shannon). The court declines to \
                 admit Rényi or Tsallis entropy as separate canonical \
                 primitives; admitted only with parameter law explicit.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID),
            reason: "Compression-ratio anomaly (Ziv-Lempel 1977 / Ziv-\
                 Lempel 1978 / Welch 1984 LZW) is ParameterizationOf \
                 (Minimum description length, 6605) with declared \
                 compression algorithm (LZ77 / LZ78 / LZW / gzip / bzip2 / \
                 xz with declared compression-level and dictionary-size \
                 parameters) + declared compression-ratio decision \
                 functional (per-window compressed-byte-count vs raw-byte-\
                 count ratio residual). Compression-ratio is a sketch-\
                 state proxy for MDL with a declared algorithm; the court \
                 declines to admit compression-ratio anomaly as a separate \
                 canonical primitive AND does NOT admit compression as a \
                 surrogate for true description length.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID),
            reason: "Learned mutual-information estimator / neural MI \
                 estimator / variational MI bound (MINE Belghazi et al. \
                 2018; InfoMax / variational MI bounds; neural KL \
                 estimators; InfoVAE; CPC contrastive predictive coding MI \
                 lower bounds) estimates MI with a trained neural network \
                 without a deterministic binning / kernel / partition law, \
                 declared formula, declared tie-break, declared training-\
                 data anchor, or declared numeric mode. The decision \
                 functional depends on opaque learned weights and does NOT \
                 satisfy the panel-locked admissibility contract. Rejected \
                 unless reduced to a Deterministic_MI_Estimator_Proxy with \
                 deterministic feature-extraction law + declared formula + \
                 declared training-data anchor (pinned dataset record-\
                 hash) + declared binning OR kernel + declared tie-break \
                 law + declared numeric mode + no learned opaque \
                 embedding, all brutally explicit in a later T.12.x. The \
                 court does NOT issue mutual-information verdicts from \
                 learned MI estimators; the rejection-shell describes what \
                 is NOT admitted.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID),
            reason: "Black-box information-theoretic anomaly score from \
                 vendor pipelines (AWS Macie information-leakage scoring; \
                 IBM Guardium DAM information-theoretic anomaly heuristics; \
                 Microsoft Purview information-leakage classifier; Symantec \
                 / Broadcom DLP entropy-based anomaly score; Cisco Talos \
                 information-theoretic threat scoring) exposes anomaly \
                 verdicts without declaring the underlying log base, \
                 smoothing rule, empty-bin law, partition function, or \
                 sample-correction law. The decision functional cannot be \
                 replayed without the contract. Rejected unless reduced to \
                 a Deterministic_IT_Score_Proxy with declared formula + \
                 binning OR partition + smoothing + sample-support bound + \
                 log base + numeric mode (either via vendor publication or \
                 user deployment-time configuration pinned in a fixed \
                 receipt). The court does NOT issue privacy-leakage / \
                 cryptographic-security / information-theoretic-security \
                 verdicts from black-box vendor IT pipelines; those terms \
                 appear here only to describe what is NOT admitted.",
        },
    ]
}

/// Nine genealogy edges proposed for the post-freeze graph.
fn information_theory_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SPECTRAL_ENTROPY_SEED_ID),
            edge_kind_wire_name: "Generalizes",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CROSS_ENTROPY_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(KL_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(MDL_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(NORMALIZED_MI_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(MDL_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Nine source refs supporting the information-theory expansion.
fn information_theory_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "shannon_1948",
            title: "A Mathematical Theory of Communication",
            year: 1948,
            venue: "Bell System Technical Journal 27(3) and 27(4) (Shannon \
                entropy + cross-entropy canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "cover_thomas_2006",
            title: "Elements of Information Theory (Second Edition)",
            year: 2006,
            venue: "Wiley-Interscience (conditional entropy + mutual \
                information canonical reference; chapter 2)",
        },
        ProposedSourceRef {
            citation_key: "rissanen_1978",
            title: "Modeling by Shortest Data Description",
            year: 1978,
            venue: "Automatica 14(5) (Minimum description length canonical \
                reference)",
        },
        ProposedSourceRef {
            citation_key: "rissanen_1986",
            title: "Stochastic Complexity and Modeling",
            year: 1986,
            venue: "Annals of Statistics 14(3) (Stochastic complexity / MDL \
                refinement reference)",
        },
        ProposedSourceRef {
            citation_key: "renyi_1961",
            title: "On Measures of Entropy and Information",
            year: 1961,
            venue: "Proceedings of the Fourth Berkeley Symposium on \
                Mathematical Statistics and Probability (Rényi entropy \
                canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "tsallis_1988",
            title: "Possible Generalization of Boltzmann-Gibbs Statistics",
            year: 1988,
            venue: "Journal of Statistical Physics 52(1-2) (Tsallis entropy \
                canonical reference)",
        },
        ProposedSourceRef {
            citation_key: "schreiber_2000",
            title: "Measuring Information Transfer",
            year: 2000,
            venue: "Physical Review Letters 85(2) (Transfer entropy canonical \
                reference; admitted only as a deterministic non-causal \
                witness)",
        },
        ProposedSourceRef {
            citation_key: "ziv_lempel_welch",
            title: "A Universal Algorithm for Sequential Data Compression \
                (1977); Compression of Individual Sequences via Variable-Rate \
                Coding (1978); A Technique for High-Performance Data \
                Compression (Welch 1984)",
            year: 1977,
            venue: "IEEE Transactions on Information Theory 23(3) / 24(5); \
                IEEE Computer 17(6) (Compression-ratio / LZ77 / LZ78 / LZW \
                canonical references)",
        },
        ProposedSourceRef {
            citation_key: "vendor_it_refs_and_mine_2018",
            title: "Learned and vendor information-theoretic pipelines (MINE \
                Belghazi et al. 2018 Mutual Information Neural Estimation; \
                AWS Macie information-leakage scoring; IBM Guardium DAM \
                information-theoretic anomaly heuristics; Microsoft Purview \
                information-leakage classifier; Symantec / Broadcom DLP \
                entropy-based anomaly score; Cisco Talos information-\
                theoretic threat scoring)",
            year: 2023,
            venue: "ICML 2018 + vendor documentation (rejection-shell \
                reference; learned MI estimators and vendor IT scores lack \
                public deterministic binning / kernel / partition / formula \
                contract)",
        },
    ]
}

/// Build the T.12.p Information Theory `DedupCourtDelta`. The
/// delta names FIVE new canonicals at 6601..=6605.
fn build_information_theory_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_p_information_theory_delta",
        vec![
            DetectorCanonicalId(SHANNON_ENTROPY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MUTUAL_INFORMATION_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CROSS_ENTROPY_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MDL_RESERVED_CANONICAL_ID),
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

/// Build the T.12.p Information Theory catch-up
/// `CorpusAmendmentProposal`. Two builds against this static
/// seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_p_information_theory_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_p_information_theory_first_proposal",
        "T.12.p files the Information Theory catch-up amendment proposal. \
         Adds FIVE genuinely new canonical information-theoretic primitives \
         (Shannon entropy shift per Shannon 1948, Conditional entropy shift \
         per Cover-Thomas 2006, Mutual information break per Cover-Thomas \
         2006 structurally distinct from SEED 9 KL divergence because MI is \
         a functional on the JOINT vs PRODUCT-OF-MARGINALS whereas KL is a \
         divergence between two declared distributions, Cross-entropy / \
         negative-log-likelihood residual per Shannon 1948 with FIXED MODEL \
         distribution parameter-pinned and frozen across the comparison \
         window, Minimum description length / coding-length residual per \
         Rissanen 1978 / 1986 with declared two-part code and declared \
         L(D | M) + L(M) decomposition) at reserved canonical ids \
         6601..=6605. Each declares estimator (plug-in / Miller-Madow / \
         James-Stein / kernel) + binning OR partition law (equal-width / \
         equal-frequency / Freedman-Diaconis / declared partition function) \
         + empty-bin law (skip / Laplace alpha / Krichevsky-Trofimov 1/2) + \
         smoothing rule + sample-support bound + log base + joint-\
         distribution contract (where applicable for conditional entropy / \
         MI) + bias-correction rule (where applicable for MI) + residual \
         definition + decision functional + confuser profile + numeric mode \
         contracts. Records THREE ExistingCanonicalAuthorityResolution \
         decisions keeping Kullback-Leibler divergence (SEED 9), Jensen-\
         Shannon divergence (SEED 32), Spectral entropy (SEED 38) canonical \
         under InformationTheory. Records TWO DomainTransferOf decisions: \
         SEED 9 KL divergence as shared information-theoretic divergence \
         ancestor (cross-entropy 6604 and JS divergence SEED 32 are \
         descendants); SEED 38 Spectral entropy as shared Shannon-entropy-\
         on-distribution ancestor (Shannon entropy shift 6601 is the \
         abstract parent admitted as a new canonical). Records FOUR \
         ParameterizationOf decisions (panel-candidate primitives that \
         collapsed on closer inspection): Normalized mutual information \
         (6606) is ParameterizationOf(MI, 6603) with declared normalisation \
         function; Transfer entropy proxy per Schreiber 2000 (6607) is \
         ParameterizationOf(MI, 6603) with declared lagged-joint contract \
         and ADMITTED ONLY AS A DETERMINISTIC NON-CAUSAL WITNESS; Rényi / \
         Tsallis entropy per Rényi 1961 / Tsallis 1988 (6608) is \
         ParameterizationOf(Shannon entropy, 6601) with declared order-\
         alpha parameter law AND declared base AND declared limit-recovery; \
         Compression-ratio anomaly per Ziv-Lempel 1977 / Ziv-Lempel 1978 / \
         Welch 1984 LZW (6609) is ParameterizationOf(MDL, 6605) with \
         declared compression algorithm (LZ77 / LZ78 / LZW / gzip / bzip2 \
         / xz). Rejects TWO information-theoretic records as \
         RejectedNotDeterministic (tenth T.12.x with two rejections, \
         following T.12.g / h / i / j / k / l / m / n / o): learned mutual-\
         information estimator (6610; MINE Belghazi et al. 2018 Mutual \
         Information Neural Estimation, InfoMax / variational MI bounds, \
         neural KL estimators, InfoVAE, CPC contrastive predictive coding MI \
         lower bounds) and black-box information-theoretic anomaly score \
         (6611; AWS Macie information-leakage scoring, IBM Guardium DAM \
         information-theoretic anomaly heuristics, Microsoft Purview \
         information-leakage classifier, Symantec / Broadcom DLP entropy-\
         based anomaly score, Cisco Talos information-theoretic threat \
         scoring). Panel-locked non-claim: T.12.p admits deterministic \
         information-theoretic witnesses: entropy, divergence, mutual-\
         information, coding-length, compression, surprise, and dependence-\
         structure evidence with declared estimator, binning, smoothing, \
         sample-support, and numeric laws. It does not admit semantic \
         meaning, causal information flow certainty, privacy leakage \
         certainty, cryptographic security claims, or learned representation \
         claims. Every CanonicalAddition / ExistingCanonicalAuthorityResolution \
         reason text declares the full contract AND avoids the panel-locked \
         forbidden terms (pinned by \
         t12_p_rejects_information_witness_without_estimator_or_binning_contract, \
         t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law, \
         t12_p_rejects_mutual_information_without_joint_distribution_contract, \
         t12_p_rejects_causal_information_flow_claim_language, \
         t12_p_rejects_privacy_or_security_claim_language, \
         t12_p_rejects_learned_embedding_information_score_without_formula \
         scanners). Does NOT mutate SEED (SEED.len() stays at 54); status = \
         Open pending review.",
        SourceClass::InformationTheory,
        build_information_theory_expansion_batch(),
        build_information_theory_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_p_information_theory",
    )
}
