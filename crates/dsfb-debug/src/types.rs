//! DSFB-Debug: core domain types — public API contract.
//!
//! Every type in this module is `Copy + Clone + Debug + PartialEq`
//! with no heap allocation. This is the load-bearing engineering
//! claim of the no_std core: the entire residual evaluation pipeline
//! moves Copy values across function boundaries. No Box, no Rc, no
//! Vec on hot paths. Every public surface accepts and returns
//! `&[T]` slices only — the observer-only contract is type-enforced.
//!
//! # Mapping to paper §5
//!
//! | Type | Paper concept | Section |
//! |------|---------------|:------:|
//! | `SignTuple` | residual signature σ(k) = (‖r‖, ṙ, r̈) | §5.3 |
//! | `GrammarState` | 4-state automaton | §5.5 |
//! | `ReasonCode` | residual policy reason | §5.5 |
//! | `MotifClass` | typed motif (32 variants) | §5.6 |
//! | `Provenance` | evidence ladder (3-tier) | §5.6 |
//! | `SemanticDisposition` | bank lookup outcome | §5.6 |
//! | `MatchConfidence` | per-episode evidence packet | §11.y |
//! | `HeuristicEntry` | bank record (motif IP claim) | §4.x |
//! | `DebugEpisode` | aggregated structural episode | §7 |
//! | `BenchmarkMetrics` | RSCR / fault-recall / FP rate | §13 |
//! | `AuditRecord` | NIST SP 800-53 AU-3 record | §15 |
//!
//! # Standards alignment
//!
//! - **NIST SP 800-53 AU-3** (audit record content): `AuditRecord`
//!   fields cover the required event_type / when / where / source /
//!   outcome content axes.
//! - **NIST SP 800-53 AU-2** (auditable events):
//!   `HeuristicEntry.primary_witness_detectors` defines the named
//!   auditable events that must fire for typed confirmation.
//! - **ISO/IEC 25010** (Analysability): typed `MotifClass` + per-
//!   episode evidence packet support the Analysability quality
//!   characteristic.
//! - **IEEE 1012-2016** (V&V): the confuser-boundary mechanism
//!   provides independent validation of typed disposition.
//!
//! # Stability guarantee
//!
//! All `pub struct` fields are additive across versions; existing
//! fields retain their type and semantics. A new field added in a
//! later phase always carries a default-friendly meaning (e.g.
//! `affinity_tiers: 0` falls back to the reason-code-derived mask;
//! `primary_witness_detectors: &[]` disables Phase 8). This
//! preserves Theorem 9 deterministic replay across upgrades.

/// Residual sign tuple σ(k) = (‖r‖, ṙ, r̈)
/// Paper §5.3
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SignTuple {
    /// ‖r(k)‖ — instantaneous deviation magnitude
    pub norm: f64,
    /// ṙ(k) — finite-difference drift rate (window-to-window)
    pub drift: f64,
    /// r̈(k) — second difference / slew (curvature of trajectory)
    pub slew: f64,
}

impl SignTuple {
    pub const ZERO: Self = Self { norm: 0.0, drift: 0.0, slew: 0.0 };
}

/// Grammar state — Paper §5.5
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrammarState {
    /// Within envelope, drift inward or bounded. Normal operation.
    Admissible = 0,
    /// Approaching envelope with sustained outward drift. Early-warning.
    Boundary = 1,
    /// Exited envelope. Structural fault / actionable incident.
    Violation = 2,
}

/// Grammar reason codes — Paper §5.5 Table
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReasonCode {
    /// Normal operation — no structural concern
    Admissible,
    /// Approaching SLO boundary — monitoring recommended
    BoundaryApproach,
    /// Memory leak, latency creep, cache degradation, dependency slowdown
    SustainedOutwardDrift,
    /// Crash, spike, exception storm, deployment regression, cascading timeout
    AbruptSlewViolation,
    /// Periodic load saturation, GC pressure, cron-triggered spikes
    RecurrentBoundaryGrazing,
    /// Confirmed SLO breach — actionable incident
    EnvelopeViolation,
    /// Outward drift followed by self-correction
    DriftWithRecovery,
    /// Transient single-step boundary touch — dismissed by persistence gate
    SingleCrossing,
}

/// Policy states — developer-facing output
/// Paper §5: Silent / Watch / Review / Escalate
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PolicyState {
    /// No motif activated; persistence or corroboration gate failed
    Silent = 0,
    /// Structural activity below escalation threshold
    Watch = 1,
    /// Motif confirmed; developer review warranted
    Review = 2,
    /// Motif confirmed + violation-class; immediate attention
    Escalate = 3,
}

/// Motif classes for software debugging — Paper §5.6.
///
/// Names are anchored to IEEE 24765 vocabulary and the
/// Avizienis–Laprie–Randell dependability tree (fault → error → failure).
/// No ad-hoc terminology: every variant decomposes into established
/// software-engineering vocabulary.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MotifClass {
    // ===== Tier-1: original 10 motifs (FrameworkDesign provenance) =======

    /// Sustained monotonic memory-consumption drift.
    /// IEEE 24765: "memory leak"; A-L-R: latent fault → error.
    MemoryLeakDrift,
    /// Step-change latency propagating across dependency chain.
    /// IEEE 24765: "fault propagation"; A-L-R: error → service-failure.
    CascadingTimeoutSlew,
    /// Abrupt shift coinciding with deployment event.
    /// IEEE 24765: "regression"; A-L-R: design fault → error.
    DeploymentRegressionSlew,
    /// Oscillatory approach to latency SLO boundary.
    /// IEEE 24765: "performance degradation"; A-L-R: marginal-state error.
    CacheDegradationGrazing,
    /// Slow positive drift in queue depth + latency.
    /// IEEE 24765: "resource exhaustion"; A-L-R: error build-up.
    ConnectionPoolExhaustionDrift,
    /// Periodic slew events from garbage collection.
    /// IEEE 24765: "stop-the-world pause"; A-L-R: transient error.
    GcPressureOscillation,
    /// Sustained positive drift in error rate.
    /// IEEE 24765: "error escalation"; A-L-R: error → multi-failure regime.
    ErrorRateEscalation,
    /// Gradual latency increase in upstream dependency.
    /// IEEE 24765: "performance degradation upstream"; A-L-R: external fault.
    DependencySlowdown,
    /// CPU/memory/disk approaching ceiling.
    /// IEEE 24765: "resource saturation"; A-L-R: latent → manifest fault.
    ResourceSaturation,
    /// Message queue depth growing monotonically.
    /// IEEE 24765: "back-pressure accumulation"; A-L-R: error build-up.
    QueueBackpressure,

    // ===== Tier-2: TADBench / TrainTicket fault cases =====================

    /// Retry-storm cascade: client retries amplify upstream load.
    /// Evidence: TADBench retry-storm fault case.
    /// IEEE 24765: "retry-induced amplification"; A-L-R: cascading error.
    RetryStormCascade,
    /// Circuit breaker open-state shift; downstream calls return immediately.
    /// Evidence: TADBench circuit-breaker fault case.
    /// IEEE 24765: "fault tolerance mechanism state change".
    CircuitBreakerOpenShift,
    /// Database lock contention with rising queue + latency.
    /// Evidence: TADBench db-lock fault case.
    /// IEEE 24765: "concurrency fault"; A-L-R: synchronisation error.
    DatabaseLockContention,
    /// Authentication failure spike (auth-backend partial outage).
    /// Evidence: TADBench auth-fail fault case.
    /// IEEE 24765: "authentication subsystem failure".
    AuthenticationFailureSpike,
    /// Step shift coinciding with version-config change.
    /// Evidence: TrainTicket-Anomaly version-config fault class.
    /// IEEE 24765: "configuration regression"; A-L-R: design-time fault.
    ConfigDriftRegression,

    // ===== Tier-3: AIOps Challenge categories =============================

    /// Packet-loss-induced error escalation (network-layer fault).
    /// Evidence: AIOps Challenge packet_loss category.
    /// IEEE 24765: "communication failure (lower layer)".
    PacketLossErrorEscalation,
    /// Network-delay-induced upstream latency inflation.
    /// Evidence: AIOps Challenge network_delay category.
    /// IEEE 24765: "communication-path performance fault".
    NetworkDelayDependencyInflation,
    /// Disk-I/O saturation; concave-up latency drift.
    /// Evidence: AIOps Challenge disk_exhaustion category.
    /// IEEE 24765: "storage subsystem saturation".
    DiskIoSaturation,
    /// CPU saturation; latency drift with rising envelope occupancy.
    /// Evidence: AIOps Challenge cpu_exhaustion category.
    /// IEEE 24765: "compute resource saturation".
    CpuSaturation,
    /// JVM heap pressure; sustained latency drift with rising variance.
    /// Refines `MemoryLeakDrift` for JVM-specific signatures.
    /// Evidence: AIOps Challenge memory_exhaustion category.
    JvmHeapPressure,
    /// JVM GC pause: distinct stop-the-world latency spikes.
    /// Refines `GcPressureOscillation` for JVM-specific signatures.
    /// Evidence: AIOps Challenge jvm_resource_exhaustion category.
    JvmGcPause,

    // ===== Tier-4: MultiDimension-Localization patterns ==================

    /// Drift propagating along the service-call graph (multi-hop).
    /// Evidence: MultiDim-Localization root-cause-graph cases.
    /// IEEE 24765: "graph-structured fault propagation".
    ServiceGraphDriftPropagation,
    /// Multi-metric correlated anomaly without dominant single signal.
    /// Evidence: MultiDim-Localization high-dim cluster cases.
    /// IEEE 24765: "compound fault signature".
    HighDimAnomalyCluster,
    /// Historically-correlated metrics decorrelate; structural regime shift.
    /// Evidence: MultiDim-Localization correlation-collapse cases.
    /// IEEE 24765: "structural model invalidation".
    MetricCorrelationCollapse,

    // ===== Tier-5: DeepTraLog log + trace fusion patterns ================

    /// Log-frequency outward drift on a service.
    /// Evidence: DeepTraLog log-volume anomalies.
    /// IEEE 24765: "diagnostic-output anomaly".
    LogVolumeAnomaly,
    /// Log timing departs from trace timing pattern.
    /// Evidence: DeepTraLog log-trace temporal-mismatch cases.
    /// IEEE 24765: "instrumentation-temporal divergence".
    LogTraceTemporalDecorrelation,
    /// Log severity distribution shifts (more WARN/ERROR proportionally).
    /// Evidence: DeepTraLog severity-shift cases.
    /// IEEE 24765: "diagnostic severity escalation".
    LogSeverityEscalation,

    // ===== Tier-6: cross-cutting structural motifs =======================

    /// Concave-up approach to a ceiling; generalises ResourceSaturation.
    /// IEEE 24765: "asymptotic resource saturation".
    SaturationTrending,
    /// Short-duration high-slew event that self-resolves.
    /// IEEE 24765: "transient-only error"; A-L-R: transient fault.
    EpisodicTransientSpike,
    /// Outward drift followed by return to baseline.
    /// Refines `ReasonCode::DriftWithRecovery` into a motif class.
    /// IEEE 24765: "self-healing transient drift".
    RegressiveDriftWithRecovery,
    /// First-time approach to the admissibility envelope without
    /// recurrence or persistent drift evidence. Catches the
    /// `ReasonCode::BoundaryApproach` reason code so it isn't an
    /// orphan in the canonical bank (validated by
    /// `tests::no_orphan_reason_codes_in_canonical_bank`).
    /// IEEE 24765: "marginal-state transient"; A-L-R: dormant fault.
    EnvelopeBoundaryApproach,
    /// Envelope breach without abrupt slew evidence — the
    /// "value stepped past the threshold but the trajectory shape
    /// shows no slew" case. Catches the
    /// `ReasonCode::EnvelopeViolation` reason code so it isn't an
    /// orphan in the canonical bank.
    /// IEEE 24765: "threshold breach (smooth)"; A-L-R: error → manifest.
    EnvelopeBreach,
}

/// Semantic disposition from heuristics bank lookup
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SemanticDisposition {
    /// Known motif matched in heuristics bank
    Named(MotifClass),
    /// Endoductive mode — structure characterized but no named match
    Unknown,
}

/// Confidence-bearing motif match result.
///
/// Returned by `HeuristicsBank::match_episode_with_confidence`. The
/// `margin` field is `(top_score - runner_up_score) / top_score`,
/// clamped to `[0.0, 1.0]`. Operators reading the margin can calibrate
/// trust:
///   - margin > 0.5 → top motif clearly dominates; act on it
///   - margin in (0.2, 0.5] → moderate confidence; surface runner-up
///   - margin <= 0.2 → top and runner-up are competitive; surface both
///   - top_score == 0.0 → no motif matched (`disposition == Unknown`)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MatchConfidence {
    pub disposition: SemanticDisposition,
    pub top_score: f64,
    pub runner_up_score: f64,
    pub runner_up_motif: Option<MotifClass>,
    pub margin: f64,
    /// Phase 3 — fraction of the matched motif's affinity tiers that
    /// actually fired in the episode range, in [0, 1]. Populated only
    /// by `match_episode_with_tier_affinity`; the legacy
    /// `match_episode_with_consensus` leaves this at 0.0. Used by the
    /// adaptive margin gate (Path 3): when `tier_consensus_factor > 0.5`,
    /// the gate is halved — strong tier evidence justifies lower
    /// margin requirement.
    pub tier_consensus_factor: f64,

    /// Phase 5.6 — explicit confuser motif declared by the matched
    /// motif's `HeuristicEntry`. May differ from `runner_up_motif` if
    /// the score-based runner-up is not the declared confuser. `None`
    /// when no confuser is declared.
    pub confuser_motif: Option<MotifClass>,

    /// Phase 5.6 — score of the declared confuser computed in the same
    /// scoring pass as the matched motif. 0.0 if no confuser is
    /// declared.
    pub confuser_score: f64,

    /// Phase 5.6 — margin of top motif over its declared confuser:
    /// `(top_score - confuser_score) / top_score`, clamped to [0, 1].
    /// Used by fusion's confuser-aware typed-confirmation gate.
    /// 0.0 if no confuser is declared.
    pub margin_vs_confuser: f64,
}

/// Provenance tag — where this interpretation came from
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Built into the initial heuristics bank by DSFB design
    FrameworkDesign,
    /// Derived from benchmark dataset observation
    DatasetObserved,
    /// Confirmed in production deployment
    FieldValidated,
}

/// Heuristics bank entry — typed, provenance-aware motif record.
///
/// The struct is `Copy + Clone + Debug + PartialEq`; all fields are
/// stack-only (`'static` string slices, primitives, enums). The bank is
/// initialised at compile time as an array of `HeuristicEntry`, so
/// every field must be `const`-friendly.
///
/// The seven original fields (`motif_class` … `slew_threshold`) preserve
/// the v0.1 wire shape. The thirteen additional fields enable
/// episode-level multi-feature scoring (`match_episode`), per-motif
/// provenance ladders (`evidence_dataset`, `evidence_dataset_doi`),
/// production-engineer dashboard hints (`dashboard_hint`), and
/// taxonomy anchors (`taxonomy_ref`). All are documented in
/// `docs/heuristics_bank.md`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HeuristicEntry {
    pub motif_class: MotifClass,
    pub reason_code: ReasonCode,
    /// NOT an attribution — a candidate interpretation hypothesis
    pub candidate_interpretation: &'static str,
    pub provenance: Provenance,
    pub recommended_action: PolicyState,
    /// Minimum drift persistence (fraction of window) to trigger
    pub drift_threshold: f64,
    /// Minimum slew magnitude to trigger
    pub slew_threshold: f64,

    // ===== additive fields (Session 3) =====
    //
    // Episode-level feature thresholds. The signal-level `lookup`
    // ignores these (so v0.1 callers see no behaviour change); the new
    // `match_episode` consults them.
    /// Minimum boundary-density (fraction of episode windows in
    /// Boundary state) for this motif to trigger.
    pub boundary_density_threshold: f64,
    /// Minimum number of contributing signals (e.g. multi-service motifs
    /// require ≥ 2). Use 1 for "any".
    pub min_correlation_count: u16,
    /// Maximum number of contributing signals (e.g. single-service
    /// motifs require ≤ 1). Use `u16::MAX` for "unbounded".
    pub max_correlation_count: u16,
    /// Minimum episode duration in windows.
    pub min_duration_windows: u16,
    /// Maximum episode duration in windows. Use `u16::MAX` for "unbounded".
    pub max_duration_windows: u16,

    /// Per-motif scoring weights for `match_episode`. Default 1.0
    /// reproduces the v0.1 unit-weighted behaviour; differential
    /// weights let one motif emphasise drift while another emphasises
    /// slew or correlation.
    pub weight_drift: f64,
    pub weight_slew: f64,
    pub weight_boundary: f64,
    pub weight_correlation: f64,
    pub weight_duration: f64,

    /// Provenance ladder — which named upstream slice motivated this
    /// entry. `"FrameworkDesign"` for hand-coded entries with no
    /// dataset evidence yet; `"<dataset_key>"` (e.g.
    /// `"tadbench_F04"`, `"aiops_challenge_packet_loss"`) for entries
    /// observed in a vendored real-data fixture.
    pub evidence_dataset: &'static str,
    /// DOI of the upstream archive cited in `evidence_dataset`. Empty
    /// string when `evidence_dataset == "FrameworkDesign"`.
    pub evidence_dataset_doi: &'static str,

    /// One-line hint for a production debug engineer reading the
    /// matched motif on a dashboard. E.g. `"Inspect jvm.memory.heap.used
    /// + gc.duration over the past hour"`.
    pub dashboard_hint: &'static str,

    /// Taxonomy anchor — IEEE 24765 term and Avizienis–Laprie–Randell
    /// node. E.g. `"IEEE 24765: 'fault propagation'; A-L-R: error →
    /// service-failure"`.
    pub taxonomy_ref: &'static str,

    /// Phase 2.5 — hand-curated tier-affinity bitmask. Each bit
    /// corresponds to one detector tier (see `TIER_BIT_*` constants in
    /// `heuristics_bank.rs`). The bank's
    /// `match_episode_with_tier_affinity` AND-s this mask against the
    /// per-cell + per-window tier-fired bitmasks to compute a
    /// motif-conditional consensus boost. A mask of `0` falls back to
    /// the reason-code-derived default in `affinity_tiers_for(...)`,
    /// preserving Phase-2 behaviour for entries without a curated mask.
    pub affinity_tiers: u32,

    /// Phase 5.6 — confuser-pair adjudication. Names the primary motif
    /// expected to compete with this one for the same residual signature
    /// (e.g., DeploymentRegressionSlew's confuser is CircuitBreakerOpenShift —
    /// both are step-shaped single-service motifs). The bank's match
    /// function explicitly tracks the confuser's score during scoring,
    /// reports `margin_vs_confuser`, and fusion gates typing on whether
    /// the candidate beats its declared confuser by `margin_vs_confuser_threshold`.
    /// `None` means no confuser is declared — episode types-confirmed
    /// based on runner-up margin alone (legacy semantics).
    pub confuser_motif: Option<MotifClass>,

    /// Phase 5.6 — minimum margin against the declared confuser for typed
    /// confirmation. Episodes that beat the runner-up but not the
    /// confuser are reported as `confuser_ambiguous` rather than typed.
    /// Default 0.10 (10% of top score). Set to 0.0 to disable.
    pub margin_vs_confuser_threshold: f64,

    /// Phase 7 — primary witness tier gate (strict semantic
    /// anti-hallucination). Subset of `affinity_tiers` that MUST fire
    /// for the motif to be a valid typing candidate. Distinct from the
    /// Phase 5.6 zero-tier filter (any affinity tier suffices); this
    /// requires SPECIFIC tiers to fire. E.g., `DeploymentRegressionSlew`
    /// witnesses = {A, B, N, X} — without scalar/Page-Hinkley/offline-
    /// CPD/Pettitt step detection actually firing, the bank refuses to
    /// type "deployment regression" even if other affinity tiers
    /// (correlation, dispersion) fired. A mask of `0` disables the gate
    /// (default — match behaves identically to Phase 5.6).
    pub primary_witness_tiers: u32,

    /// Phase 8 — per-detector primary witnesses (strict ensemble-methods
    /// SOTA gate). Named-detector subset that MUST fire for the motif to
    /// type-confirm. Distinct from `primary_witness_tiers` (any detector
    /// in the named tiers suffices); this requires SPECIFIC detectors
    /// (e.g., `[poisson_burst, burst_after_silence]` for
    /// AuthenticationFailureSpike). Names match the `detector_name`
    /// field returned by detector functions in `incumbent_baselines.rs`.
    /// Empty slice `&[]` disables the gate (default — Phase 7 behaviour).
    /// Applied at fusion.rs typed-confirmation, not at bank match time;
    /// failing motifs are demoted to ambiguous rather than skipped, so
    /// the runner-up motif can still be reported in the operator packet.
    pub primary_witness_detectors: &'static [&'static str],
}

/// Per-signal, per-window DSFB evaluation result.
/// The atomic unit of the traceability chain.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SignalEvaluation {
    pub window_index: u64,
    pub signal_index: u16,
    pub residual_value: f64,
    pub sign_tuple: SignTuple,
    pub raw_grammar_state: GrammarState,
    /// After hysteresis confirmation (n_confirm=2)
    pub confirmed_grammar_state: GrammarState,
    pub reason_code: ReasonCode,
    pub motif: Option<MotifClass>,
    pub semantic_disposition: SemanticDisposition,
    pub dsa_score: f64,
    pub policy_state: PolicyState,
    /// Missingness-aware flag: if true, drift=0, slew=0, grammar=Admissible
    pub was_imputed: bool,
    /// Drift persistence at this evaluation (fraction of last `drift_window`
    /// windows with positive drift). Persisted so that episode-level
    /// `match_episode` can average across an episode's window range
    /// without recomputing norm histories.
    pub drift_persistence: f64,
}

/// Drift direction in an episode's structural signature
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DriftDirection {
    Positive,
    Negative,
    Oscillatory,
    None,
}

/// Structural signature of an episode
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StructuralSignature {
    pub dominant_drift_direction: DriftDirection,
    pub peak_slew_magnitude: f64,
    pub duration_windows: u64,
    pub signal_correlation: f64,
}

/// A debugging episode — the Trace Event Collapse output.
/// Paper §7: "the primary developer-facing delta"
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DebugEpisode {
    pub episode_id: u32,
    pub start_window: u64,
    pub end_window: u64,
    pub peak_grammar_state: GrammarState,
    pub primary_reason_code: ReasonCode,
    pub matched_motif: SemanticDisposition,
    pub policy_state: PolicyState,
    pub contributing_signal_count: u16,
    pub structural_signature: StructuralSignature,
    /// Most-upstream contributing signal in the service-call graph,
    /// if a graph was supplied to `run_evaluation_with_graph` and a
    /// unique upstream root could be determined; else `None`.
    /// Populated by `causality::attribute_root_cause`.
    pub root_cause_signal_index: Option<u16>,
}

/// NIST 800-53 AU-3 compliant audit record
/// Fields: what (event_type), when (window_index), where (signal_index),
///         source (source), outcome (outcome)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AuditRecord {
    pub event_type: AuditEventType,
    pub window_index: u64,
    pub signal_index: u16,
    pub source: AuditSource,
    pub outcome: PolicyState,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuditEventType {
    GrammarStateTransition,
    EpisodeOpened,
    EpisodeClosed,
    PolicyEscalation,
    MotifMatched,
    EndoductiveUnknown,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuditSource {
    GrammarEvaluator,
    EpisodeAggregator,
    PolicyEngine,
    HeuristicsBank,
}

/// Benchmark metrics — the paper's headline numbers
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BenchmarkMetrics {
    pub dataset_name: &'static str,
    pub total_windows: u64,
    pub total_signals: u16,
    pub raw_anomaly_count: u64,
    pub dsfb_episode_count: u64,
    /// Review Surface Compression Ratio = raw / episodes
    pub rscr: f64,
    /// Fraction of episodes preceding labeled faults within W_pred
    pub episode_precision: f64,
    /// Fraction of labeled faults captured by at least one episode
    pub fault_recall: f64,
    pub investigation_load_raw: u64,
    pub investigation_load_dsfb: u64,
    pub investigation_load_reduction_pct: f64,
    /// Negative control: false episode rate in clean windows
    pub clean_window_false_episode_rate: f64,
}
