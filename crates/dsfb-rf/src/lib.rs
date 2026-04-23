//! # dsfb-rf — DSFB Structural Semiotics Engine for RF Signal Monitoring
//!
//! **What this crate is, in one paragraph.** A deterministic, `no_std`,
//! zero-`unsafe` *observer* that reads residual streams — PLL innovation,
//! AGC error, channel-equaliser residual, matched-filter discrepancy,
//! GNSS tracking-loop residual, scheduler EWMA, beamformer weight-update
//! innovation — which existing RF receivers already compute, and structures
//! them into a typed grammar of human-readable episodes. DSFB does not
//! replace matched filters, Kalman/Luenberger observers, CFAR, or ML
//! classifiers; it *augments* them by giving operators a structural view of
//! what those systems discard. Removing DSFB leaves the upstream receiver
//! chain unchanged.
//!
//! See the forward-looking programmatic framing below.
//!
//! ---
//!
//! **Invariant Forge LLC** — Prior art under 35 U.S.C. § 102.
//! Commercial deployment requires a separate written license.
//! Reference implementation: Apache-2.0.
//! <licensing@invariantforge.net>
//!
//! ## Sovereign Spectrum Governance
//!
//! **dsfb-rf** implements *Sovereign Spectrum Governance* — a command-and-control
//! (C2) architecture for the RF electromagnetic domain that elevates spectrum
//! management from a reactive utility service to a proactive, structurally-aware
//! C2 capability.
//!
//! Traditional spectrum management detects events after thresholds are crossed.
//! Sovereign Spectrum Governance detects **the trajectory toward events** before
//! they occur, providing governance actors with actionable structural intelligence
//! at the physics timescale — not the incident-response timescale.
//!
//! The four pillars of Sovereign Spectrum Governance in this crate:
//!
//! 1. **Structural anticipation** — grammar episodes begin before envelope
//!    violations (Theorem 1). The governance actor is notified of drift
//!    direction before any threshold is crossed.
//! 2. **Traceable authority** — every policy decision is backed by a deterministic
//!    audit chain (`dsfb_traceability.json`) from raw IQ residual to policy output.
//!    No black-box inference, no probabilistic ambiguity in the C2 chain.
//! 3. **Non-interference sovereignty** — the observer contract guarantees that
//!    the governance layer does not modify, degrade, or interfere with the
//!    governed RF system. Sovereignty is exercised through observation alone.
//! 4. **Distributed corroboration** — BFT swarm consensus (`swarm_consensus.rs`)
//!    and baseline sanity checks (`calibration::swarm_baseline_sanity_check`)
//!    prevent adversarial contamination of the governance baseline.
//!
//! ## Overview
//!
//! This crate implements the DSFB Structural Semiotics Engine for RF signal
//! monitoring as described in:
//!
//! > de Beer, R. (2026). *DSFB-RF Structural Semiotics Engine for RF Signal
//! > Monitoring — A Deterministic, Non-Intrusive Observer Layer for Typed
//! > Structural Interpretation of IQ Residual Streams in Electronic Warfare,
//! > Spectrum Monitoring, and Cognitive Radio* (v1.0). Invariant Forge LLC.
//! > Zenodo. DOI: [10.5281/zenodo.19702330](https://doi.org/10.5281/zenodo.19702330)
//!
//! The engine is a **read-only, non-intrusive, deterministic observer layer**
//! that sits above existing RF receiver infrastructure (matched-filter banks,
//! CFAR detectors, AGC loops, PLLs, spectrum analyzers) and interprets the
//! IQ residual streams those systems already produce.
//!
//! ## Semiotic Decimation for High-Rate Deployment
//!
//! At 1 GSPS and above, the full semiotic pipeline can be time-budgeted via
//! the `DecimationAccumulator` (`engine::DecimationAccumulator`):
//!
//! ```rust,no_run
//! use dsfb_rf::engine::{DsfbRfEngine, DecimationAccumulator};
//! use dsfb_rf::platform::PlatformContext;
//! // 1 GSPS, 100 kHz structural monitoring rate
//! let mut eng = DsfbRfEngine::<10, 4, 8>::new(0.1, 2.0)
//!     .with_decimation(10_000);
//! // observe_decimated() returns None for 9999 samples, Some on the 10000th
//! let ctx = PlatformContext::operational();
//! let result = eng.observe_decimated(0.05, ctx);
//! // DSFB monitors the *envelope* of the physics, not the cycle of the carrier.
//! ```
//!
//! ## Architectural Contract
//!
//! - **Observer-only**: the `observe()` method takes `&self` and `&[f32]`
//!   (immutable references only). There is no mutable write path into any
//!   upstream data structure.
//! - **`#![no_std]`**: the core modules link against neither the Rust standard
//!   library nor any OS runtime. Deployable on bare-metal FPGA softcores
//!   (RISC-V, Cortex-M4F) without an OS or heap allocator.
//! - **`#![no_alloc]`**: all internal structures use fixed-capacity
//!   array-backed types. No heap allocation in any hot path.
//! - **Zero `unsafe`**: no `unsafe` blocks, no `UnsafeCell`, no `RefCell`
//!   in any observer code path. Enforced at compile time by
//!   `#![forbid(unsafe_code)]` (see `src/lib.rs` crate attribute).
//!
//! ## Non-Claims (from paper §11)
//!
//! This crate does **not** provide:
//! - Emitter classification or modulation recognition
//! - Calibrated Pd/Pfa guarantees
//! - Hard real-time latency bounds under FPGA/DSP deployment
//! - Adversarial robustness against spoofing or smart jamming
//! - ITAR determination or export-control assessment
//! - MIL-STD-461G, DO-178C, or 3GPP TS 36.141 compliance
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | *(none)* | Core engine: `no_std` + `no_alloc` + zero unsafe |
//! | `alloc` | Opt-in heap via `alloc` crate for host-side tooling |
//! | `std` | Opt-in std library for pipeline and output modules |
//! | `serde` | JSON artifact serialization (requires `std`) |
//! | `paper_lock` | Headline metric enforcement for reproducibility |
//! | `experimental` | Exploratory extensions not validated in the companion paper (`tda`, `quantum_noise`, `rg_flow`). Excluded from the paper-lock metric set. |
//!
//! ## Usage (bare-metal, no_std)
//!
//! ```rust,no_run
//! use dsfb_rf::{DsfbRfEngine, PolicyDecision};
//! use dsfb_rf::platform::PlatformContext;
//!
//! // W=10 drift window, K=4 persistence, M=8 heuristics bank capacity
//! let mut engine = DsfbRfEngine::<10, 4, 8>::new(0.1_f32, 2.0_f32);
//!
//! let residual_norm: f32 = 0.045; // |r(k)| from your receiver
//! let ctx = PlatformContext::operational();
//! let result = engine.observe(residual_norm, ctx);
//! let decision: PolicyDecision = result.policy;
//! // decision: Silent | Watch | Review | Escalate
//! // upstream receiver: UNCHANGED
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---------------------------------------------------------------
// Conditional std/alloc imports
// ---------------------------------------------------------------
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// ---------------------------------------------------------------
// Core modules — unconditionally no_std + no_alloc + zero unsafe
// ---------------------------------------------------------------

/// Residual sign tuple: (‖r‖, ṙ, r̈) — the semiotic manifold coordinate.
pub mod math;
pub mod sign;

/// Admissibility envelope E(k) = {r : ‖r‖ ≤ ρ(k)}.
pub mod envelope;

/// Grammar FSM: Admissible | Boundary[reason] | Violation.
pub mod grammar;

/// Syntax layer: classify sign tuples into named temporal motifs.
pub mod syntax;

/// Heuristics bank H: fixed-capacity typed RF motif library.
pub mod heuristics;

/// Deterministic Structural Accumulator (DSA) score.
pub mod dsa;

/// Lyapunov stability analysis: finite-time Lyapunov exponents for
/// residual trajectory divergence quantification.
pub mod lyapunov;

/// Wide-Sense Stationarity (WSS) verification for calibration windows.
/// Applies the Wiener-Khinchin pre-condition before envelope radius is set.
pub mod stationarity;

/// Information-theoretic complexity estimation (MDL/Kolmogorov framing).
/// Quantifies how well the nominal model describes the current residual trajectory.
pub mod complexity;

/// Zero-copy residual source trait for DMA buffer integration.
/// Enables tap into memory-mapped IQ buffers without CPU copy overhead.
pub mod zero_copy;

/// Industry standards: VITA 49.2 VRT, SigMF annotations, MIL-STD-461G masks,
/// SOSA/MORA alignment structs, and 3GPP/ITU-R envelope mappings.
pub mod standards;

/// GUM-compliant uncertainty budget (ISO/IEC Guide 98-3) for envelope ρ.
/// Type A (statistical) + Type B (systematic) uncertainty decomposition.
pub mod uncertainty;

/// Physics-of-failure mapping and semiotic horizon characterization.
/// Maps grammar states to candidate physical mechanisms (Arrhenius, Leeson, etc.).
pub mod physics;

/// Policy engine: Silent | Watch | Review | Escalate.
pub mod policy;

/// Platform context: waveform transitions, SNR floor, regime suppression.
pub mod platform;

/// Main engine: composes all stages into a single deterministic observer.
pub mod engine;

/// Hierarchical Residual-Envelope Trust (HRET) for multi-channel RF receivers.
///
/// Two-level EMA-based channel + group trust weights, derived from DSFB-HRET
/// (de Beer 2026).  The hierarchical composition ŵ_k = w_k · w_{g[k]} / Σ
/// provides optimal channel down-weighting when partial antenna failures,
/// polarisation-group faults, or localised RFI degrade a subset of channels.
pub mod trust;

/// Regime-switched admissibility envelopes and grammar trust scalars.
///
/// Fixed / Widening / Tightening / RegimeSwitched / Aggregate envelope modes
/// for burst-mode receivers, PLL acquisition transients, and multi-constraint
/// aggregate bounds.  Geometry-based grammar trust scalar T ∈ [0, 1] from the
/// DSFB-Semiotics-Engine (§IV), with abrupt-slew and recurrent-grazing detection.
pub mod regime;

/// Rich deterministic detectability taxonomy (DSFB-Lattice §IV–V).
///
/// `DetectabilityClass` / `SemanticStatus` / `DetectionStrengthBand` hierarchy
/// with post-crossing persistence tracking and the deterministic τ_upper = δ₀/(α−κ)
/// detection latency bound.  Provides a full operator-advisory output for
/// VITA 49.2 context packets and SIGINT operator dashboards.
pub mod detectability;

/// Q16.16 fixed-point ingress path for FPGA and bare-metal deployment.
///
/// `quantize_q16_16` / `dequantize_q16_16`, saturation arithmetic.
/// Enables deployment on RISC-V RV32I, Cortex-M0, and custom FPGA pipelines
/// without a hardware FPU.  Mode label `"fixed_q16_16"` for SigMF provenance.
pub mod fixedpoint;

/// RF disturbance taxonomy: DDMF-derived classification with RF mapping.
///
/// PointwiseBounded / Drift / SlewRateBounded / Impulsive / PersistentElevated
/// disturbance classes, envelope adaptation flags, recommended envelope mode
/// per class, and a heuristic structural classifier that infers typed disturbance
/// hypotheses from the grammar / Lyapunov / DSA pipeline outputs.
pub mod disturbance;

/// Hardware and channel impairment injection for Continuous Rigor pipelines.
///
/// Deterministic, physics-grounded perturbation functions (I/Q imbalance,
/// DC offset, PA compression, ADC quantisation noise, phase noise,
/// ionospheric scintillation, Doppler tracking error) applied in the physical
/// signal-chain order.  Parameterised by named hardware profiles (RTL-SDR,
/// USRP X310, Colosseum node, ESA L-band receiver).  Used by the Stage II
/// "Impairment Injection" step of every benchmark example.
pub mod impairment;

/// Continuous Rigor 4-stage audit report for benchmark examples.
///
/// `StageResult` captures per-stage detection statistics (FA rate, lead time,
/// λ_peak).  `AuditReport` consolidates all three stages plus the SigMF-
/// derived ground-truth comparison.  `print()` emits a formatted console
/// report meeting SBIR Phase II documentation standards.
pub mod audit;

/// Delay-coordinate attractor embedding, correlation dimension D₂, and Koopman
/// operator proxy for RF residual streams.
///
/// Takens (1981) embedding theorem + Grassberger-Procaccia (1983) D₂ estimator.
/// Distinguishes stochastic residual balls from structured low-dimensional orbits
/// (hidden determinism, cyclostationary jammers, LO injection locking artifacts).
pub mod attractor;

/// Topological Data Analysis (TDA) of residual windows.
///
/// **Exploratory extension — not validated in the companion paper (de Beer 2026).
/// Excluded from the paper-lock metric set. Enable with `--features experimental`
/// for research use; not part of the stable public API.**
///
/// Betti₀ via union-find Rips filtration + persistence landscape + topological
/// innovation score.  Edelsbrunner et al. (2002) persistent homology.
/// Detects topological phase transitions in interference environments.
#[doc(hidden)]
pub mod tda;

/// Pragmatic information gating for SOSA/MORA backplane traffic shaping.
///
/// Atlan-Cohen (1998) pragmatic information criterion: only emit grammar
/// observations that change the receiver's belief state by ≥ Δh.
/// Achieves > 99% backplane suppression in Admissible steady state while
/// preserving 100% of state-transition events.
pub mod pragmatic;

/// Hardware DNA authentication via Allan variance fingerprinting.
///
/// Oscillator-intrinsic σ_y(τ) fingerprint at τ = 1…128 samples.
/// Cosine-similarity matching for physical-layer authentication against
/// hardware swap attacks and clock-injection spoofing.
/// Allan (1966); IEEE Std 1139-2008; Danev et al. (2010).
pub mod dna;

/// Calibration sensitivity sweeps: ρ-perturbation, W_pred grid, config landscape.
///
/// Fills the deferred analyses promised in §14.6 (ρ ± 15 % sensitivity),
/// §14.7 (W_pred optimisation grid), and §18.4 (hyperparameter landscape).
/// Constants are anchored to Table IV nominal operating point
/// (87 episodes, 73.6 % precision, 95.1 % recall).
/// All arithmetic uses `math::exp_f32` — no libm dependency.
pub mod calibration;

/// Waveform transition schedule for grammar-escalation suppression.
///
/// Near-term engineering extension per §18.3 (Waveform Transition Artifacts).
/// Registers deliberate transition windows (frequency hops, burst boundaries,
/// power ramps) so the grammar layer can suppress spurious `Violation`
/// escalation during known waveform-change intervals.
/// Fixed-capacity (`WaveformSchedule<N>`) — no_std, no_alloc.
pub mod waveform_context;

/// Landauer thermodynamic cost estimation for computational inference operations.
///
/// Quantifies the minimum thermodynamic cost (Joules) implied by each
/// observation under Landauer's Principle — k_B·T·ln2 per bit erasure.
/// Provides `LandauerAudit` per window with `LandauerClass` severity tiers
/// and a structural energy budget for power-constrained FPGA deployments.
/// Landauer (1961); Bennett & Rolf (1985).
pub mod energy_cost;

/// Fisher-Rao information geometry on the Gaussian statistical manifold.
///
/// Computes geodesic distances between Gaussian states (μ, σ) under the
/// Fisher metric — the unique Riemannian distance invariant under sufficient
/// statistics transformations.  `ManifoldTracker` accumulates the cumulative
/// geodesic path length as the receiver's residual distribution drifts,
/// providing a geometry-native drift indicator independent of signal units.
/// Atkinson & Mitchell (1981); Calvo & Oller (1990).
pub mod fisher_geometry;

/// Relativistic Doppler corrections for hypersonic and exo-atmospheric platforms.
///
/// Provides exact relativistic Doppler factors (not the classical v/c
/// approximation) for β = v/c up to Mach 1000.  Quantifies the residual
/// frequency error from using a classical Doppler corrector on a relativistic
/// platform.  Corrections become significant only for β > 1e-5 (≈ Mach 100);
/// below that threshold this module returns classical predictions unmodified.
/// NON-CLAIM: plasma sheath effects (Mach 5–10) are not modelled here.
/// Relativistic Doppler corrections for hypersonic, LEO, and high-Doppler-rate platforms.
///
/// The primary use case is **NOT** Mach 30 scenarios. The dominant practical
/// driver is high-Doppler-rate environments (LEO satellite handovers at
/// 7.8 km/s, high-speed drone maneuvers at 50 g lateral acceleration) where
/// d(f_D)/dt exceeds 2nd-order PLL tracking bandwidth and produces lag-drift
/// residuals indistinguishable from structural events without this correction.
/// This module is a passive safety guard: `correction_required() -> false`
/// for 99.9% of deployments; zero overhead when not activated.
pub mod high_dynamics;

/// Quantum-limited noise floor digital twin for Rydberg and cryogenic SDRs.
///
/// Quantum-limited noise floor digital twin for Rydberg and cryogenic SDRs.
///
/// **Exploratory extension — not validated in the companion paper (de Beer 2026).
/// Excluded from the paper-lock metric set. Enable with `--features experimental`
/// for research use; not part of the stable public API.**
///
/// Implements the Standard Quantum Limit (SQL) noise floor model:
/// ħωB for shot noise, k_B·T·B for thermal noise, and their ratio R_QT.
/// `QuantumNoiseTwin` tracks the regime (DeepThermal / TransitionRegime /
/// QuantumLimited / BelowSQL) as a function of carrier, bandwidth, and
/// physical temperature.  NON-CLAIM: all current commercial SDRs are in
/// DeepThermal regime; QuantumLimited requires millikelvin cryogenics.
#[doc(hidden)]
pub mod quantum_noise;

/// Byzantine-Fault-Tolerant (BFT) semiotic consensus from distributed DSFB-RF nodes.
///
/// Aggregates `GrammarVote` reports from up to 64 swarm nodes using:
/// (1) hardware-authentication gating, (2) Kolmogorov-Smirnov outlier
/// rejection of Byzantine votes, and (3) BFT quorum (2f+1) weighted tally.
/// Produces a `SwarmConsensus` report with admission statistics and
/// cross-node modal grammar state.
pub mod swarm_consensus;

/// Renormalization Group (RG) flow on TDA persistence diagrams.
///
/// **Exploratory extension — not validated in the companion paper (de Beer 2026).
/// Excluded from the paper-lock metric set. Enable with `--features experimental`
/// for research use; not part of the stable public API.**
///
/// Iteratively coarse-grains persistence features to distinguish local
/// hardware flukes (features that vanish after 1–2 RG steps) from systemic
/// environment changes (features that survive all scale levels).
/// Based on Wilson & Kogut (1974) RG flow applied to Edelsbrunner-Harer
/// persistence filtrations.  Produces `RgFlowClass` with escalation guidance.
#[doc(hidden)]
pub mod rg_flow;

// ---------------------------------------------------------------
// std-gated modules
// ---------------------------------------------------------------

/// Pipeline: host-side Stage III evaluation (requires `std`).
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod pipeline;

/// Output: artifact serialization and traceability chain (requires `serde`).
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod output;

/// Paper-lock: headline metric enforcement for reproducibility (requires `paper_lock`).
#[cfg(feature = "paper_lock")]
#[cfg_attr(docsrs, doc(cfg(feature = "paper_lock")))]
pub mod paper_lock;

/// RadioML 2018.01a HDF5 dataset loader for real-data Stage III evaluation.
///
/// Loads the DeepSig RadioML 2018.01a HDF5 file (`X`/`Z` schema), computes
/// per-capture RMS amplitude, and builds the [`pipeline::RfObservation`] and
/// [`pipeline::RegimeTransitionEvent`] streams required by
/// [`pipeline::run_stage_iii`]. Requires `libhdf5` on the host system.
///
/// Enabled with `--features hdf5_loader`. Not part of the `no_std` core.
#[cfg(feature = "hdf5_loader")]
#[cfg_attr(docsrs, doc(cfg(feature = "hdf5_loader")))]
pub mod hdf5_loader;

/// GNU Radio sink block for read-only IQ residual tap integration.
///
/// Implements the `dsfb_sink_b200` tap architecture described in paper §II.B:
/// a parallel read-only GNU Radio sink that taps the CF32 stream at the
/// channel filter output, runs the DSFB grammar, and emits SigMF-formatted
/// episode metadata on a ZeroMQ socket — with **zero modification** to the
/// upstream flowgraph (demodulator, CFAR, AGC, USRP firmware).
///
/// Supports: USRP B200/X310, LimeSDR, RTL-SDR (any UHD/SoapySDR-compatible
/// platform that exposes a CF32 stream). Phase I deliverable: USRP B200
/// integration in 30 days; A/B verification of zero upstream impact.
///
/// Requires `std` (GNU Radio integration is host-side only).
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod sink_gnuradio;

/// Kani formal verification harnesses (panic-freedom proofs for the FSM).
///
/// Activated only when running `cargo kani`; completely excluded from all
/// normal builds. See [`kani_proofs`] module documentation for the full
/// proof inventory, running instructions, and empirical honesty statements.
///
/// # Kani Coverage (panel §XIX evidence)
///
/// | Harness | Property Proved |
/// |---|---|
/// | `proof_grammar_evaluator_no_panic` | GrammarEvaluator::evaluate() never panics under arbitrary f32 |
/// | `proof_grammar_state_severity_bounded` | severity() ∈ {0,1,2} and severity_trust() ∈ [0,1] |
/// | `proof_envelope_judgment_consistency` | is_violation() ⊂ is_boundary_approach() |
/// | `proof_decimation_exact_epoch_count` | DecimationAccumulator fires exactly once per factor samples |
/// | `proof_fixedpoint_resync_drift_bounded` | post-resync drift ≤ max_drift_ulps |
/// | `proof_quantize_q16_16_no_panic` | quantize_q16_16() never panics on finite f32 |
#[cfg(kani)]
pub mod kani_proofs;

// ---------------------------------------------------------------
// Public re-exports — flat API surface for ergonomics
// ---------------------------------------------------------------
pub use engine::{DsfbRfEngine, NonIntrusiveContract, NON_INTRUSIVE_CONTRACT};
pub use grammar::{GrammarState, ReasonCode};
pub use heuristics::{HeuristicsBank, MotifEntry, Provenance, SemanticDisposition};
pub use syntax::MotifClass;
pub use policy::PolicyDecision;
pub use sign::SignTuple;
pub use envelope::AdmissibilityEnvelope;
pub use dsa::DsaScore;
pub use platform::{PlatformContext, WaveformState, SnrFloor};
pub use lyapunov::{LyapunovEstimator, LyapunovResult, StabilityClass};

// New module re-exports
pub use trust::{HretEstimator, HretParams, HretResult};
pub use regime::{RegimeEnvelope, RegimeEnvelopeParams, EnvelopeMode, RfRegime,
                 GrammarTrustScalar, EnvelopeUpdateResult};
pub use detectability::{DetectabilityClass, SemanticStatus, DetectionStrengthBand,
                        DetectabilityBound, DetectabilitySummary, DetectabilityTracker,
                        DetectabilityThresholds};
pub use fixedpoint::{quantize_q16_16, dequantize_q16_16, q16_16_to_f32, quantize_f32,
                     mul_q16_16, add_q16_16, MODE_LABEL};
pub use disturbance::{RfDisturbance, DisturbanceLog, DisturbanceHypothesis,
                      DisturbanceClassifier};
pub use impairment::{ImpairmentVector, ScintillationClass,
                     apply_iq_imbalance, apply_dc_offset, apply_pa_compression,
                     quantization_noise_std, apply_phase_noise,
                     apply_scintillation, classify_s4, doppler_residual_floor,
                     apply_all as apply_impairments,
                     lcg_step, lcg_uniform, sin_approx, cos_approx};
pub use audit::{StageResult, AuditReport, SigMfAnnotation};

// New Phase-4 module re-exports
pub use attractor::{DelayEmbedding, AttractorResult, NoiseAttractorState};
// tda re-exports are gated: see experimental feature below
pub use pragmatic::{PragmaticGate, PragmaticConfig};
pub use dna::{AllanVarianceEstimator, HardwareDna, DnaMatchResult, DnaVerdict,
              cosine_similarity, verify_dna, AUTHENTICATION_THRESHOLD, ALLAN_TAUS};
pub use uncertainty::CrlbFloor;
pub use physics::{PhysicsModel, ArrheniusModel, AllanVarianceModel,
                  PhysicsConsistencyResult, evaluate_physics_consistency};
pub use stationarity::{
    reverse_arrangements_test, ReverseArrangementsResult,
    BootstrapIntegrityAlert, check_bootstrap_integrity,
    StationarityVerdict, StationarityConfig, verify_wss,
};
pub use complexity::{PermutationEntropyEstimator, PermEntropyResult, PermEntropyRegime};

// Phase-5 re-exports
pub use calibration::{
    run_rho_sweep, run_wpred_grid, run_config_grid, check_calibration_window,
    RhoSweepResult, RhoSweepCell, WpredGrid, WpredCell,
    ConfigGrid, ConfigCell, CalibWindowIntegrity,
    NOM_EPISODES, NOM_PRECISION, NOM_RECALL, NOM_TP, NOM_FP,
};
pub use waveform_context::{
    WaveformSchedule, TransitionWindow, TransitionKind, SuppressionDecision,
    suppress_escalation,
    freq_hop_window, burst_start_window, power_change_window,
};
pub use math::exp_f32;

// Phase-6 re-exports
pub use math::ln_f32;
pub use energy_cost::{
    LandauerAudit, LandauerClass, landauer_audit,
    structural_energy_joules, structural_power_watts,
    thermal_noise_power, gaussian_entropy_nats, K_BOLTZMANN,
};
pub use fisher_geometry::{
    GaussPoint, ManifoldTracker, DriftGeometry,
    fisher_rao_distance, fisher_rao_distance_exact, geodesic_curvature,
};
pub use high_dynamics::{
    LorentzFactor, HighDynamicsSettings, high_dynamics_settings,
    relativistic_doppler_hz, classical_doppler_hz,
    relativistic_correction_residual_hz, C_LIGHT_M_S, MACH_1_SEA_LEVEL_M_S,
};
// quantum_noise re-exports are gated: see experimental feature below
pub use swarm_consensus::{
    GrammarVote, SwarmConsensus, compute_consensus, consensus_grammar_state,
    GovernanceTag, NodeGovernanceReport, swarm_governance_report,
    MAX_SWARM_NODES,
};
// rg_flow re-exports are gated: see experimental feature below
pub use heuristics::{KnownClockClass, classify_clock_instability};

// ---------------------------------------------------------------
// Exploratory-extension re-exports (require `experimental` feature)
// These modules implement research-stage capabilities not validated in
// the companion paper (de Beer 2026). They are excluded from the
// paper-lock metric set and the Stage III fixed evaluation protocol.
// A hostile reviewer will find #[doc(hidden)] on the module declarations
// and these cfg-gated re-exports — not a stable API claim.
// ---------------------------------------------------------------
#[cfg(feature = "experimental")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental")))]
pub use tda::{UnionFind, TopologicalState, PersistenceEvent, detect_topological_innovation};

#[cfg(feature = "experimental")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental")))]
pub use quantum_noise::{
    QuantumNoiseTwin, ReceiverRegime,
    shot_noise_power_w, thermal_noise_power_w,
    quantum_to_thermal_ratio, thermal_photon_number,
};

#[cfg(feature = "experimental")]
#[cfg_attr(docsrs, doc(cfg(feature = "experimental")))]
pub use rg_flow::{
    RgFlowResult, RgFlowClass, RgScale, compute_rg_flow,
    MAX_RG_SCALES, MAX_RG_EVENTS,
};

// Phase-7 re-exports: Semiotic Decimation, Sovereign Baselines, Robust Manifold, FP Resync
pub use engine::DecimationAccumulator;
pub use calibration::{swarm_baseline_sanity_check, BaselineConsensusAlert};
pub use fisher_geometry::{RobustManifoldMode, GaussPointRobust};
pub use fixedpoint::{PeriodicResyncConfig, apply_periodic_resync};
