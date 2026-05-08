//! # dsfb-debug — Structural Semiotics Engine for Software Debugging
//!
//! A deterministic, read-only, observer-only augmentation layer that
//! turns the residuals every observability stack already discards into
//! typed, human-readable debugging episodes with full evidence trails.
//!
//! ## Augmentation, not replacement
//!
//! DSFB-Debug does NOT compete with existing observability tools
//! (Datadog, OpenTelemetry, Jaeger, Sentry, Prometheus, ELK). It sits
//! on top of them as a passive observer that ingests their residuals
//! and produces typed structural interpretation. The intended deployment
//! is: existing tools detect anomalies; DSFB-Debug structures them
//! into typed episodes; operators receive the union as actionable
//! insight rather than alert noise.
//!
//! ## Non-Intrusion Contract (type-enforced)
//!
//! Every public function accepts only shared immutable references
//! (`&[T]`). There is NO mutable write path into any upstream data
//! structure. The Rust type system enforces this at compile time.
//! See `docs/non_intrusion_contract.md` for the formal contract.
//!
//! ## Crate Properties (compile-time enforced)
//!
//! - `#![no_std]` — no standard library dependency in the core
//! - `#![forbid(unsafe_code)]` — zero unsafe blocks anywhere
//! - `#![deny(clippy::unwrap_used)]` — no panic paths
//! - Zero runtime Cargo dependencies — SHA-256, DFT, matrix algebra
//!   are hand-rolled in `src/adapters/` and `src/incumbent_baselines.rs`
//! - Deterministic — identical inputs always produce byte-identical
//!   outputs (Theorem 9, formally proven in paper §6.4)
//!
//! ## Feature Gates
//!
//! - `std` — enables `Vec`-based variable-size buffers and the
//!   adapter layer. The no_std core is unchanged.
//! - `paper-lock` — enables `evaluate_real_dataset`, the
//!   `RealDatasetManifest` struct, and SHA-256 integrity gating.
//!   Implies `std`. Without `paper-lock`, the real-data entry point
//!   is physically absent from the compiled artefact.
//!
//! ## Pipeline Architecture
//!
//! ```text
//! Residual → SignTuple → Grammar → Hysteresis → ReasonCode → Bank lookup → Episode
//! ```
//!
//! The engine is a pipeline of deterministic stages. Each stage has
//! its own module:
//!
//! | Stage | Module | Output type |
//! |-------|--------|-------------|
//! | Residual extraction | `adapters/residual_projection.rs` | `OwnedResidualMatrix` |
//! | Sign tuple | `sign.rs` | `SignTuple` |
//! | Grammar | `grammar.rs` | `GrammarState` |
//! | Hysteresis | `policy.rs` | confirmed `GrammarState` |
//! | Reason code | `policy.rs` | `ReasonCode` |
//! | Heuristics bank lookup | `heuristics_bank.rs` | `SemanticDisposition` |
//! | Episode aggregation | `episode.rs` | `DebugEpisode` |
//! | Multi-detector fusion | `fusion.rs` | `FusionMetrics` (std-only) |
//! | Causality attribution | `causality.rs` | `root_cause_signal_index` |
//! | Operator rendering | `render.rs` | `String` (std-only) |
//!
//! ## Standards Alignment
//!
//! - **NIST SP 800-53 Rev. 5**: AU-2 (auditable events: named
//!   primary witness detectors), AU-3 (record content: per-motif
//!   provenance + DOI + taxonomy), AU-6 (review/analysis: episode
//!   catalog), AU-12 (audit generation: deterministic replay)
//! - **NIST SP 800-92**: §4.2 (log analysis), §5 (log management)
//! - **NIST SP 800-171 Rev. 2**: §3.3 (Audit & Accountability)
//! - **DO-178C** §6.3: certification-pathway-eligible architectural
//!   foresight (NOT a certification claim)
//! - **IEEE 1012-2016** §7: V&V verification tool classification
//! - **ISO/IEC 25010:2023**: Analysability, Testability
//! - **OpenTelemetry Semantic Conventions**: OTLP-compatible residual ingestion
//! - **W3C Trace Context Level 1**: §3 (traceparent), §4 (tracestate)
//! - **SOC 2 Type II**: CC7.2, CC7.3 (Monitoring Activities)
//!
//! ## Theorem 9 (Deterministic Replay)
//!
//! For any byte-stable residual matrix input, two consecutive
//! `engine.run_evaluation(...)` calls produce byte-identical episode
//! output. Mechanically proven by composition of deterministic stages
//! (paper §6.4); empirically verified on every real-bytes vendored
//! fixture by `verify_deterministic_replay`. Failure of this assertion
//! on real bytes surfaces as a hard test failure, not a silent
//! metric drift.

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

// `std` feature opts the std-only adapter and real-dataset entry point in.
// The no_std core (residual / sign / grammar / policy / episode pipeline)
// is byte-stable regardless of which features are enabled.
#[cfg(feature = "std")]
extern crate std;

pub mod types;
pub mod error;
pub mod config;
pub mod residual;
pub mod sign;
pub mod envelope;
pub mod grammar;
pub mod heuristics_bank;
pub mod dsa;
pub mod policy;
pub mod episode;
pub mod baseline;
pub mod causality;
pub mod graph_inference;
pub mod episode_catalog;

// std-only adapters and real-dataset entry point. Absent from the no_std
// default build.
#[cfg(feature = "std")]
pub mod adapters;
#[cfg(feature = "paper-lock")]
pub mod real_data;
#[cfg(feature = "std")]
pub mod calibration;
#[cfg(feature = "std")]
pub mod incumbent_baselines;
#[cfg(feature = "std")]
pub mod render;
#[cfg(feature = "std")]
pub mod fusion;

#[cfg(feature = "std")]
pub mod audit;

#[cfg(feature = "demo")]
pub mod demo;

use types::*;
use error::{DsfbError, Result};
use config::EngineConfig;
use heuristics_bank::HeuristicsBank;

/// Main DSFB debugging engine — stateless, deterministic, read-only.
///
/// The engine evaluates telemetry residuals through the DSFB pipeline
/// and produces typed, auditable debugging episodes.
///
/// # Const Generic Parameters
/// - `MAX_SIGNALS`: maximum number of monitored signals (span durations, error rates, etc.)
/// - `MAX_MOTIFS`: maximum heuristics bank entries
///
/// # Non-Intrusion Contract
///
/// All public methods accept only `&self` and `&[T]` (shared immutable references).
/// There is NO mutable write path into any upstream data structure.
/// The Rust type system enforces this at compile time.
pub struct DsfbDebugEngine<
    const MAX_SIGNALS: usize,
    const MAX_MOTIFS: usize,
> {
    config: EngineConfig,
    heuristics_bank: HeuristicsBank<MAX_MOTIFS>,
}

impl<const S: usize, const M: usize> DsfbDebugEngine<S, M> {
    /// Create a new engine with the given configuration.
    pub fn new(config: EngineConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            heuristics_bank: HeuristicsBank::with_canonical_motifs(),
        })
    }

    /// Create a new engine with paper-lock configuration.
    pub fn paper_lock() -> Result<Self> {
        Self::new(config::PAPER_LOCK_CONFIG)
    }

    /// Get the engine configuration (read-only)
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get the engine's heuristics bank (read-only). Exposed for the
    /// fusion harness's consensus-aware scoring pass; operators
    /// reading this can call `bank.match_episode_with_consensus(...)`
    /// against any closed `DebugEpisode` directly.
    pub fn heuristics_bank(&self) -> &HeuristicsBank<M> {
        &self.heuristics_bank
    }

    /// Evaluate a single window of telemetry for a single signal.
    ///
    /// # Non-Intrusion Contract
    /// All inputs are shared immutable references. Cannot modify upstream data.
    ///
    /// # Arguments
    /// * `residual_norms` - historical residual norms for this signal (immutable)
    /// * `k` - current window index into the norms array
    /// * `rho` - envelope radius for this signal
    /// * `signal_index` - index of this signal
    /// * `window_index` - global window index
    /// * `was_imputed` - whether this observation was imputed (missing data)
    /// * `recent_raw_states` - last n_confirm raw grammar states
    /// * `persistence_count` - consecutive Boundary windows
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_signal(
        &self,
        residual_norms: &[f64],     // immutable
        k: usize,
        rho: f64,
        signal_index: u16,
        window_index: u64,
        was_imputed: bool,
        recent_raw_states: &[GrammarState], // immutable
        persistence_count: usize,
    ) -> SignalEvaluation {
        // Missingness-aware: imputed signals → zero everything
        if was_imputed {
            return SignalEvaluation {
                window_index,
                signal_index,
                residual_value: 0.0,
                sign_tuple: SignTuple::ZERO,
                raw_grammar_state: GrammarState::Admissible,
                confirmed_grammar_state: GrammarState::Admissible,
                reason_code: ReasonCode::Admissible,
                motif: None,
                semantic_disposition: SemanticDisposition::Unknown,
                dsa_score: 0.0,
                policy_state: PolicyState::Silent,
                was_imputed: true,
                drift_persistence: 0.0,
            };
        }

        // Step 1: Compute sign tuple
        let sign_tuple = sign::compute_sign_tuple(residual_norms, k);

        // Step 2: Compute drift persistence
        let drift_pers = sign::drift_persistence(
            residual_norms, k, self.config.drift_window,
        );

        // Step 3: Evaluate raw grammar state
        let (raw_grammar, reason_code) = grammar::evaluate_raw_grammar(
            &sign_tuple, rho, &self.config, drift_pers,
        );

        // Step 4: Apply hysteresis confirmation
        let confirmed = grammar::hysteresis_confirm(
            recent_raw_states, self.config.hysteresis_confirm,
        );
        // Use the higher of hysteresis result and current raw (Violation bypasses)
        let confirmed_grammar = if raw_grammar == GrammarState::Violation {
            GrammarState::Violation
        } else {
            confirmed
        };

        // Step 5: Compute DSA features
        // Simplified: use drift persistence as primary DSA input
        let slew_mag = if sign_tuple.slew > 0.0 { sign_tuple.slew } else { -sign_tuple.slew };
        let dsa_score = dsa::compute_dsa_score(
            0.0, // boundary density would need state history — simplified
            drift_pers,
            if slew_mag > self.config.slew_delta { 1.0 } else { 0.0 },
        );
        let gate_passed = dsa::consistency_gate(dsa_score, self.config.consistency_gate);

        // Step 6: Heuristics bank lookup (semantics)
        let semantic = self.heuristics_bank.lookup(reason_code, drift_pers, slew_mag);

        // Step 7: Extract motif class
        let motif = match semantic {
            SemanticDisposition::Named(m) => Some(m),
            SemanticDisposition::Unknown => None,
        };

        // Step 8: Apply policy
        let policy_state = policy::apply_policy(
            confirmed_grammar,
            dsa_score,
            gate_passed,
            semantic,
            persistence_count,
            self.config.persistence_threshold,
        );

        SignalEvaluation {
            window_index,
            signal_index,
            residual_value: if k < residual_norms.len() { residual_norms[k] } else { 0.0 },
            sign_tuple,
            raw_grammar_state: raw_grammar,
            confirmed_grammar_state: confirmed_grammar,
            reason_code,
            motif,
            semantic_disposition: semantic,
            dsa_score,
            policy_state,
            was_imputed: false,
            drift_persistence: drift_pers,
        }
    }

    /// Run the full DSFB evaluation pipeline over a dataset.
    ///
    /// This is the main entry point for benchmark evaluation.
    ///
    /// # Non-Intrusion Contract
    /// All input slices are shared immutable references.
    /// Outputs are written into caller-owned mutable buffers.
    ///
    /// # Arguments
    /// * `data` - row-major observation data [window][signal] (immutable)
    /// * `num_signals` - signals per window
    /// * `num_windows` - total windows
    /// * `fault_labels` - per-window fault labels (immutable)
    /// * `healthy_window_end` - index of last healthy window for baseline
    /// * `eval_out` - output buffer for per-signal evaluations (row-major)
    /// * `episodes_out` - output buffer for episodes
    /// * `dataset_name` - name for metrics reporting
    ///
    /// # Returns
    /// (episode_count, BenchmarkMetrics)
    #[allow(clippy::too_many_arguments)]
    pub fn run_evaluation(
        &self,
        data: &[f64],                    // immutable
        num_signals: usize,
        num_windows: usize,
        fault_labels: &[bool],           // immutable
        healthy_window_end: usize,
        eval_out: &mut [SignalEvaluation],
        episodes_out: &mut [DebugEpisode],
        dataset_name: &'static str,
    ) -> Result<(usize, BenchmarkMetrics)> {
        if num_signals > S {
            return Err(DsfbError::SignalBufferFull);
        }
        if data.len() < num_windows * num_signals {
            return Err(DsfbError::DimensionMismatch {
                expected: num_windows * num_signals,
                got: data.len(),
            });
        }
        // Flat-aggregation buffers below are sized FLAT_CAP. Refuse rather
        // than silently truncate the policy/reason/drift/slew streams.
        const FLAT_CAP: usize = 8192;
        let needed = match num_signals.checked_mul(num_windows) {
            Some(n) => n,
            None => return Err(DsfbError::BufferTooSmall { needed: usize::MAX, available: FLAT_CAP }),
        };
        if needed > FLAT_CAP {
            return Err(DsfbError::BufferTooSmall { needed, available: FLAT_CAP });
        }

        // Phase 1: Compute baseline from healthy window
        let mut baseline_mean = [0.0_f64; S];
        let mut rho = [0.0_f64; S];
        let healthy_data_end = healthy_window_end * num_signals;
        let healthy_slice = if healthy_data_end <= data.len() {
            &data[..healthy_data_end]
        } else {
            data
        };
        baseline::compute_baseline_mean(
            healthy_slice, num_signals, healthy_window_end, &mut baseline_mean[..num_signals],
        );
        baseline::compute_baseline_envelope(
            healthy_slice, &baseline_mean[..num_signals],
            num_signals, healthy_window_end, &mut rho[..num_signals],
        );

        // Phase 2: Compute residuals and evaluate each signal at each window
        // We need per-signal norm histories for sign tuple computation
        // Use a rolling buffer approach — keep norms inline

        // Flatten evaluation: track per-signal state
        let mut persistence_counts = [0_usize; S];
        let mut recent_raw = [[GrammarState::Admissible; 4]; S]; // last 4 raw states per signal
        let mut raw_head = [0_usize; S]; // circular index

        // Per-signal norm histories for drift computation
        // We'll compute norms incrementally and store in eval_out for traceability
        let mut policy_states_flat: [PolicyState; 8192] = [PolicyState::Silent; 8192];
        let mut reason_codes_flat: [ReasonCode; 8192] = [ReasonCode::Admissible; 8192];
        let mut drift_dirs_flat: [DriftDirection; 8192] = [DriftDirection::None; 8192];
        let mut slew_mags_flat: [f64; 8192] = [0.0; 8192];
        let mut raw_anomaly_count: u64 = 0;

        // We need per-signal norm arrays. Since no_alloc, use a fixed window.
        // Keep last (drift_window + 2) norms per signal for sign computation.
        const NORM_HIST: usize = 32; // enough for any reasonable drift_window
        let mut norm_histories = [[0.0_f64; NORM_HIST]; S];
        let mut norm_heads = [0_usize; S];

        let mut w = 0_usize;
        while w < num_windows {
            let mut s = 0_usize;
            while s < num_signals {
                let data_idx = w * num_signals + s;
                let obs = if data_idx < data.len() { data[data_idx] } else { 0.0 };
                let is_nan = obs.is_nan(); // NaN check
                let residual = if is_nan { 0.0 } else { obs - baseline_mean[s] };
                let norm = residual::residual_norm(residual);

                // Push norm into per-signal history
                let h = norm_heads[s];
                if h < NORM_HIST {
                    norm_histories[s][h] = norm;
                    norm_heads[s] = h + 1;
                } else {
                    // Shift left (simple, O(NORM_HIST) but NORM_HIST is small)
                    let mut i = 0;
                    while i < NORM_HIST - 1 {
                        norm_histories[s][i] = norm_histories[s][i + 1];
                        i += 1;
                    }
                    norm_histories[s][NORM_HIST - 1] = norm;
                }

                let nh = norm_heads[s];
                let k = if nh > 0 { nh - 1 } else { 0 };

                // Build recent_raw_states slice for hysteresis
                let rh = raw_head[s];
                let recent_slice_len = if rh < 4 { rh } else { 4 };
                let _recent_start = rh.saturating_sub(4);
                // We need a contiguous slice — use the array directly
                let recent = &recent_raw[s][..recent_slice_len];

                let eval = self.evaluate_signal(
                    &norm_histories[s][..nh],
                    k,
                    rho[s],
                    s as u16,
                    w as u64,
                    is_nan,
                    recent,
                    persistence_counts[s],
                );

                // Update persistence count
                if eval.confirmed_grammar_state >= GrammarState::Boundary {
                    persistence_counts[s] += 1;
                } else {
                    persistence_counts[s] = 0;
                }

                // Update raw state history (circular)
                let rh_idx = raw_head[s] % 4;
                recent_raw[s][rh_idx] = eval.raw_grammar_state;
                raw_head[s] += 1;

                // Store evaluation
                let eval_idx = w * num_signals + s;
                if eval_idx < eval_out.len() {
                    eval_out[eval_idx] = eval;
                }

                // Store flattened arrays for episode aggregation
                let flat_idx = w * num_signals + s;
                if flat_idx < policy_states_flat.len() {
                    policy_states_flat[flat_idx] = eval.policy_state;
                    reason_codes_flat[flat_idx] = eval.reason_code;
                    slew_mags_flat[flat_idx] = if eval.sign_tuple.slew > 0.0 {
                        eval.sign_tuple.slew
                    } else {
                        -eval.sign_tuple.slew
                    };
                    drift_dirs_flat[flat_idx] = if eval.sign_tuple.drift > 0.1 {
                        DriftDirection::Positive
                    } else if eval.sign_tuple.drift < -0.1 {
                        DriftDirection::Negative
                    } else {
                        DriftDirection::None
                    };
                }

                // Count raw anomalies (any signal in Boundary or Violation)
                if eval.confirmed_grammar_state >= GrammarState::Boundary {
                    raw_anomaly_count += 1;
                }

                s += 1;
            }
            w += 1;
        }

        // Phase 3: Episode aggregation (Trace Event Collapse)
        let total_flat = num_windows * num_signals;
        let flat_len = if total_flat < policy_states_flat.len() {
            total_flat
        } else {
            policy_states_flat.len()
        };

        let episode_count = episode::aggregate_episodes(
            &policy_states_flat[..flat_len],
            num_signals,
            num_windows,
            &reason_codes_flat[..flat_len],
            &drift_dirs_flat[..flat_len],
            &slew_mags_flat[..flat_len],
            self.config.episode_correlation_window,
            episodes_out,
        );

        // Phase 3b: Episode-level heuristics-bank match (Session 3).
        // For every closed episode, compute average drift persistence
        // and average boundary density over the episode's window range
        // from `eval_out`, then call `match_episode` to populate the
        // previously-stub `matched_motif` field with a real disposition.
        let mut ep_idx: usize = 0;
        while ep_idx < episode_count {
            let ep = episodes_out[ep_idx];
            let start_w = ep.start_window as usize;
            let end_w = ep.end_window as usize;

            let mut sum_drift: f64 = 0.0;
            let mut boundary_count: usize = 0;
            let mut total: usize = 0;
            let mut w = start_w;
            while w <= end_w && w < num_windows {
                let mut s = 0;
                while s < num_signals {
                    let idx = w * num_signals + s;
                    if idx < eval_out.len() {
                        let e = eval_out[idx];
                        sum_drift += e.drift_persistence;
                        if e.confirmed_grammar_state == GrammarState::Boundary {
                            boundary_count += 1;
                        }
                        total += 1;
                    }
                    s += 1;
                }
                w += 1;
            }
            let avg_drift = if total > 0 { sum_drift / total as f64 } else { 0.0 };
            let avg_boundary = if total > 0 { boundary_count as f64 / total as f64 } else { 0.0 };

            let disposition = self.heuristics_bank.match_episode(&ep, avg_drift, avg_boundary);
            // Write the disposition back into the episode output buffer.
            episodes_out[ep_idx].matched_motif = disposition;
            // If the disposition resolved to a Named motif, also reflect it
            // through the `policy_state` selection: violations stay
            // Escalate; boundary episodes inherit the bank's recommended
            // action where it is more conservative (no downgrade below
            // Review for boundary episodes).
            if let SemanticDisposition::Named(motif) = disposition {
                let recommended = self.heuristics_bank.recommended_action(motif);
                if episodes_out[ep_idx].policy_state == PolicyState::Review
                    && recommended == PolicyState::Escalate
                {
                    episodes_out[ep_idx].policy_state = PolicyState::Escalate;
                }
            }

            ep_idx += 1;
        }

        // Phase 4: Compute metrics
        let metrics = episode::compute_metrics(
            episodes_out,
            episode_count,
            fault_labels,
            raw_anomaly_count,
            self.config.episode_precision_window,
            dataset_name,
            num_signals as u16,
        );

        Ok((episode_count, metrics))
    }

    /// `run_evaluation` plus graph-attribution: same return value, but
    /// each closed episode's `root_cause_signal_index` is populated by
    /// walking the supplied service-call graph (see `crate::causality`).
    ///
    /// Backward-compatible with v0.1: callers without a graph use
    /// `run_evaluation` and get `root_cause_signal_index = None` on
    /// every episode.
    #[allow(clippy::too_many_arguments)]
    pub fn run_evaluation_with_graph(
        &self,
        data: &[f64],
        num_signals: usize,
        num_windows: usize,
        fault_labels: &[bool],
        healthy_window_end: usize,
        eval_out: &mut [SignalEvaluation],
        episodes_out: &mut [DebugEpisode],
        dataset_name: &'static str,
        service_graph: &[(u16, u16)],
    ) -> Result<(usize, BenchmarkMetrics)> {
        let (episode_count, metrics) = self.run_evaluation(
            data, num_signals, num_windows, fault_labels,
            healthy_window_end, eval_out, episodes_out, dataset_name,
        )?;
        causality::attribute_root_causes(
            episodes_out,
            episode_count,
            eval_out,
            num_signals,
            num_windows,
            service_graph,
            self.config.slew_delta,
        );
        Ok((episode_count, metrics))
    }

    /// Deterministic replay verification (Theorem 9 proof-by-construction).
    ///
    /// Runs the evaluation twice on identical inputs and verifies identical outputs.
    pub fn verify_deterministic_replay(
        &self,
        data: &[f64],
        num_signals: usize,
        num_windows: usize,
        fault_labels: &[bool],
        healthy_window_end: usize,
    ) -> Result<bool> {
        // First run
        let mut eval1 = [SignalEvaluation {
            window_index: 0, signal_index: 0, residual_value: 0.0,
            sign_tuple: SignTuple::ZERO,
            raw_grammar_state: GrammarState::Admissible,
            confirmed_grammar_state: GrammarState::Admissible,
            reason_code: ReasonCode::Admissible,
            motif: None, semantic_disposition: SemanticDisposition::Unknown,
            dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
            drift_persistence: 0.0,
        }; 4096];
        let blank_ep = DebugEpisode {
            episode_id: 0, start_window: 0, end_window: 0,
            peak_grammar_state: GrammarState::Admissible,
            primary_reason_code: ReasonCode::Admissible,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: PolicyState::Silent,
            contributing_signal_count: 0,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::None,
                peak_slew_magnitude: 0.0, duration_windows: 0, signal_correlation: 0.0,
            },
            root_cause_signal_index: None,
        };
        let mut ep1 = [blank_ep; 256];
        let (c1, m1) = self.run_evaluation(
            data, num_signals, num_windows, fault_labels,
            healthy_window_end, &mut eval1, &mut ep1, "replay_test",
        )?;

        // Second run — identical inputs
        let mut eval2 = eval1;
        // Reset eval2
        let mut i = 0;
        while i < eval2.len() {
            eval2[i] = SignalEvaluation {
                window_index: 0, signal_index: 0, residual_value: 0.0,
                sign_tuple: SignTuple::ZERO,
                raw_grammar_state: GrammarState::Admissible,
                confirmed_grammar_state: GrammarState::Admissible,
                reason_code: ReasonCode::Admissible,
                motif: None, semantic_disposition: SemanticDisposition::Unknown,
                dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
                drift_persistence: 0.0,
            };
            i += 1;
        }
        let mut ep2 = [blank_ep; 256];
        let (c2, m2) = self.run_evaluation(
            data, num_signals, num_windows, fault_labels,
            healthy_window_end, &mut eval2, &mut ep2, "replay_test",
        )?;

        // Verify identical outputs
        if c1 != c2 { return Ok(false); }
        if m1.dsfb_episode_count != m2.dsfb_episode_count { return Ok(false); }
        if m1.raw_anomaly_count != m2.raw_anomaly_count { return Ok(false); }

        // Check episode-level equality
        let mut j = 0;
        while j < c1 {
            if ep1[j] != ep2[j] { return Ok(false); }
            j += 1;
        }

        Ok(true)
    }
}

// Default implementation for common use case (Session 3: MAX_MOTIFS bumped 32 → 64).
impl DsfbDebugEngine<256, 64> {
    /// Create with default const generics (256 signals, 64 motifs).
    ///
    /// The 64-slot heuristics bank holds 29 canonical motifs as of v0.2
    /// (Session 3 expansion); the remaining 35 slots provide v0.3 / v0.4
    /// headroom for additional site-specific findings.
    pub fn default_size() -> Result<Self> {
        Self::paper_lock()
    }
}
