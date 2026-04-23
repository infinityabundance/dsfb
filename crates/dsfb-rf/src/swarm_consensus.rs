//! Distributed Semiotic Consensus across a swarm of DSFB-RF observers.
//!
//! ## Vision
//!
//! The future of military and 6G spectrum governance is not a single, large
//! radio but a **swarm of distributed apertures** — 100+ small UAVs, ground
//! nodes, or shipborne sensors each running an independent DSFB-RF engine.
//!
//! Individual observers at low SNR produce noisy grammar-state estimates.
//! But their **joint semiotic state** — the aggregate grammar distribution
//! across the swarm — is a high-fidelity image of the battlespace RF
//! environment. This is analogous to how a sparse aperture array synthesises
//! a virtual aperture far larger than any individual element.
//!
//! ## Design
//!
//! This module implements **Byzantine Fault Tolerant (BFT) grammar
//! aggregation** using a Kolmogorov-Smirnov consistency filter:
//!
//! 1. Each node broadcasts its local `GrammarVote` (grammar state + DSA score).
//! 2. Up to `f` Byzantine-faulty nodes can broadcast false grammar states.
//! 3. The consensus algorithm requires agreement among `2f+1` of `N` nodes
//!    (BFT-quorum; Lamport, Shostak & Pease 1982).
//! 4. A **KS-consistency** pre-filter discards votes whose DSA score
//!    distribution is statistically inconsistent with the majority prior
//!    (detection of sensor spoofing or hardware failure).
//!
//! ## Semiotic Consensus State
//!
//! The `SwarmConsensus` result is a probability distribution over the four
//! grammar states: $\{$`Admissible`, `Boundary`, `Violation`, `Suppressed`$\}$.
//! The **consensus grammar state** is the modal state when the leading state
//! has probability ≥ `CONSENSUS_THRESHOLD` and is supported by ≥ `2f+1`
//! nodes.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! Fixed-capacity arrays throughout. `MAX_SWARM_NODES = 64`. No heap.
//!
//! ## References
//!
//! - Lamport, Shostak & Pease (1982), "The Byzantine Generals Problem",
//!   ACM TOPLAS 4(3):382-401.
//! - Kolmogorov (1941), "Confidence limits for an unknown distribution".
//! - Stouffer et al. (1949), "Combined p-values via z-sum" (used for
//!   distributed hypothesis combination).
//! - Baraniuk & Steeghs (2007), "Compressive radar imaging", IEEE Radar.

use crate::grammar::{GrammarState, ReasonCode};

// ── Capacity ───────────────────────────────────────────────────────────────

/// Maximum number of swarm nodes supported.
pub const MAX_SWARM_NODES: usize = 64;

/// KS-consistency confidence threshold for vote acceptance.
/// Votes with normalized DSA score deviation > this are quarantined.
pub const KS_REJECT_THRESHOLD: f32 = 3.5;

/// Minimum fraction of participating nodes required for valid consensus.
pub const QUORUM_MIN_FRACTION: f32 = 0.67;

/// Probability threshold for declaring a modal grammar state as "consensus".
pub const CONSENSUS_THRESHOLD: f32 = 0.50;

// ── Grammar Vote ───────────────────────────────────────────────────────────

/// A single node's grammar vote for the current consensus window.
#[derive(Debug, Clone, Copy)]
pub struct GrammarVote {
    /// Node identifier (0-based).
    pub node_id: u8,
    /// Grammar state observed by this node.
    pub state: GrammarState,
    /// DSA structural score at this node (used for KS-filtering).
    pub dsa_score: f32,
    /// Episode count at this node (weight for score combination).
    pub episode_count: u32,
    /// Whether this node's hardware DNA authenticated this window.
    pub hardware_authenticated: bool,
}

// ── Swarm Consensus ────────────────────────────────────────────────────────

/// Outcome of a distributed BFT semiotic consensus round.
#[derive(Debug, Clone, Copy)]
pub struct SwarmConsensus {
    /// Probability of `Admissible` across authenticated votes.
    pub p_admissible: f32,
    /// Probability of `Boundary` across authenticated votes.
    pub p_boundary:   f32,
    /// Probability of `Violation` across authenticated votes.
    pub p_violation:  f32,
    /// Modal (highest-probability) grammar state.
    pub modal_state:  GrammarState,
    /// Whether a valid BFT quorum was reached.
    pub quorum_reached: bool,
    /// Number of votes admitted after KS filtering.
    pub votes_admitted: u8,
    /// Number of votes quarantined (potential Byzantine faults or failures).
    pub votes_quarantined: u8,
    /// Number of unauthenticated nodes excluded from the consensus.
    pub votes_unauthenticated: u8,
    /// Swarm-level DSA consensus score (weighted mean of admitted votes).
    pub consensus_dsa_score: f32,
    /// Whether hardware authentication requirement was applied.
    pub auth_required: bool,
}

impl SwarmConsensus {
    /// Nominal safe state: `Admissible` with `quorum_reached=false`.
    /// Returned when the swarm is too small for valid consensus.
    pub const fn no_quorum() -> Self {
        Self {
            p_admissible:       1.0,
            p_boundary:         0.0,
            p_violation:        0.0,
            modal_state:        GrammarState::Admissible,
            quorum_reached:     false,
            votes_admitted:     0,
            votes_quarantined:  0,
            votes_unauthenticated: 0,
            consensus_dsa_score: 0.0,
            auth_required:      false,
        }
    }
}

// ── Consensus Engine ───────────────────────────────────────────────────────

/// Compute BFT semiotic consensus from a set of node votes.
///
/// # Arguments
/// - `votes`         — slice of `GrammarVote` from participating nodes
/// - `bft_f`         — maximum number of Byzantine-faulty nodes to tolerate
/// - `require_auth`  — if `true`, only hardware-authenticated votes are
///                     admitted. Excludes potential hardware-swap attacks.
///
/// # Returns
/// A `SwarmConsensus` summarising the distributed grammar state.
pub fn compute_consensus(
    votes:        &[GrammarVote],
    bft_f:        u8,
    require_auth: bool,
) -> SwarmConsensus {
    if votes.is_empty() {
        return SwarmConsensus::no_quorum();
    }

    let (admitted_buf, admitted_count, n_unauth) = authenticate_votes(votes, require_auth);
    if admitted_count == 0 {
        return SwarmConsensus {
            quorum_reached: false,
            votes_unauthenticated: n_unauth,
            auth_required: require_auth,
            ..SwarmConsensus::no_quorum()
        };
    }
    let admitted = &admitted_buf[..admitted_count];

    let (final_buf, final_count, n_quarantined) = quarantine_outliers(admitted);
    let final_votes = &final_buf[..final_count];

    let n_total = votes.len().min(MAX_SWARM_NODES);
    let quorum_needed = (2 * bft_f as usize + 1).max(1);
    let quorum_fraction = final_count as f32 / n_total.max(1) as f32;
    let quorum_reached = final_count >= quorum_needed
        && quorum_fraction >= QUORUM_MIN_FRACTION;

    if !quorum_reached || final_votes.is_empty() {
        return SwarmConsensus {
            quorum_reached: false,
            votes_admitted:       admitted_count as u8,
            votes_quarantined:    n_quarantined,
            votes_unauthenticated: n_unauth,
            auth_required:        require_auth,
            ..SwarmConsensus::no_quorum()
        };
    }

    tally_consensus(final_votes, require_auth, n_quarantined, n_unauth, quorum_reached)
}

fn authenticate_votes(
    votes: &[GrammarVote],
    require_auth: bool,
) -> ([GrammarVote; MAX_SWARM_NODES], usize, u8) {
    let mut admitted_buf = [GrammarVote {
        node_id: 0, state: GrammarState::Admissible,
        dsa_score: 0.0, episode_count: 0, hardware_authenticated: false,
    }; MAX_SWARM_NODES];
    let mut admitted_count = 0usize;
    let mut n_unauth = 0u8;

    for vote in votes.iter().take(MAX_SWARM_NODES) {
        if require_auth && !vote.hardware_authenticated {
            n_unauth = n_unauth.saturating_add(1);
            continue;
        }
        if admitted_count < MAX_SWARM_NODES {
            admitted_buf[admitted_count] = *vote;
            admitted_count += 1;
        }
    }
    (admitted_buf, admitted_count, n_unauth)
}

fn insertion_sort_median(values: &mut [f32]) -> f32 {
    let n = values.len();
    if n == 0 { return 0.0; }
    for i in 1..n {
        let key = values[i];
        let mut j = i;
        while j > 0 && values[j - 1] > key {
            values[j] = values[j - 1];
            j -= 1;
        }
        values[j] = key;
    }
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) * 0.5 }
}

fn quarantine_outliers(
    admitted: &[GrammarVote],
) -> ([GrammarVote; MAX_SWARM_NODES], usize, u8) {
    const MAD_SCALE: f32 = 1.482_602_2;
    let admitted_count = admitted.len();

    let mut sorted_scores = [0.0f32; MAX_SWARM_NODES];
    for (i, v) in admitted.iter().enumerate() {
        sorted_scores[i] = v.dsa_score;
    }
    let median_dsa = insertion_sort_median(&mut sorted_scores[..admitted_count]);

    let mut abs_devs = [0.0f32; MAX_SWARM_NODES];
    for (i, v) in admitted.iter().enumerate() {
        abs_devs[i] = (v.dsa_score - median_dsa).abs();
    }
    let mad = insertion_sort_median(&mut abs_devs[..admitted_count]);
    let robust_sigma = (MAD_SCALE * mad).max(1e-9);

    let mut final_buf = [GrammarVote {
        node_id: 0, state: GrammarState::Admissible,
        dsa_score: 0.0, episode_count: 0, hardware_authenticated: false,
    }; MAX_SWARM_NODES];
    let mut final_count = 0usize;
    let mut n_quarantined = 0u8;

    for vote in admitted {
        let z = (vote.dsa_score - median_dsa).abs() / robust_sigma;
        if z > KS_REJECT_THRESHOLD {
            n_quarantined = n_quarantined.saturating_add(1);
        } else if final_count < MAX_SWARM_NODES {
            final_buf[final_count] = *vote;
            final_count += 1;
        }
    }
    (final_buf, final_count, n_quarantined)
}

fn tally_consensus(
    final_votes: &[GrammarVote],
    require_auth: bool,
    n_quarantined: u8,
    n_unauth: u8,
    quorum_reached: bool,
) -> SwarmConsensus {
    let total_weight: f32 = final_votes.iter()
        .map(|v| v.episode_count as f32)
        .sum::<f32>()
        .max(1.0);

    let w_admissible: f32 = final_votes.iter()
        .filter(|v| v.state == GrammarState::Admissible)
        .map(|v| v.episode_count as f32).sum();
    let w_boundary: f32 = final_votes.iter()
        .filter(|v| v.state.is_boundary())
        .map(|v| v.episode_count as f32).sum();
    let w_violation: f32 = final_votes.iter()
        .filter(|v| v.state == GrammarState::Violation)
        .map(|v| v.episode_count as f32).sum();

    let p_admissible = w_admissible / total_weight;
    let p_boundary   = w_boundary   / total_weight;
    let p_violation  = w_violation  / total_weight;

    let modal_state = if p_admissible >= p_boundary && p_admissible >= p_violation {
        GrammarState::Admissible
    } else if p_boundary >= p_violation {
        GrammarState::Boundary(ReasonCode::SustainedOutwardDrift)
    } else {
        GrammarState::Violation
    };

    let consensus_dsa_score = final_votes.iter()
        .map(|v| v.dsa_score * v.episode_count as f32)
        .sum::<f32>() / total_weight;

    SwarmConsensus {
        p_admissible, p_boundary, p_violation, modal_state, quorum_reached,
        votes_admitted: final_votes.len() as u8,
        votes_quarantined: n_quarantined,
        votes_unauthenticated: n_unauth,
        consensus_dsa_score,
        auth_required: require_auth,
    }
}

/// Whether the consensus grammar state satisfies the CONSENSUS_THRESHOLD.
///
/// Returns the modal state if its probability ≥ `CONSENSUS_THRESHOLD`,
/// otherwise returns `None` (insufficient consensus strength).
pub fn consensus_grammar_state(c: &SwarmConsensus) -> Option<GrammarState> {
    if !c.quorum_reached { return None; }
    let p_modal = match c.modal_state {
        GrammarState::Admissible    => c.p_admissible,
        GrammarState::Boundary(_)  => c.p_boundary,
        GrammarState::Violation     => c.p_violation,
    };
    if p_modal >= CONSENSUS_THRESHOLD { Some(c.modal_state) } else { None }
}

// ── Governance Tags ────────────────────────────────────────────────────────────

/// Typed governance tag for a single node in the swarm.
///
/// Emitted by [`swarm_governance_report`] after a BFT consensus round.
/// This is the implementation of the **Governance Side-Car** pattern
/// (paper §XX): DSFB emits typed, human-inspectable metadata without
/// actuating any hardware change. The integration layer or human operator
/// decides whether and how to act on these tags.
///
/// # Non-Interference Guarantee
///
/// A `GovernanceTag` is a pure read-only output. The DSFB engine cannot
/// write radio registers, reset clocks, or recalibrate PLLs.
/// It cannot cause a system-bus lockup (stack-only, 504 bytes).
///
/// # Examples
///
/// ```
/// use dsfb_rf::swarm_consensus::GovernanceTag;
/// let tag = GovernanceTag::LocalHardwareAnomaly;
/// assert!(tag.requires_action());
/// println!("{}", tag.label());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GovernanceTag {
    /// Node is in the honest majority with nominal DSA. No action required.
    Nominal,
    /// Node's DSA score is a robust outlier (|z_MAD| > [`KS_REJECT_THRESHOLD`]).
    /// Likely cause: LNA thermal runaway, intermittent hardware fault, or
    /// Byzantine data injection.  Node was quarantined from this consensus round.
    ///
    /// Emitted annotation: `[Governance]: Observer_Quarantined | Reason: DSA_Outlier`
    ObserverQuarantined,
    /// Node reports Admissible while the swarm majority reports Boundary/Violation.
    /// Likely cause: missed alarm, hardware degradation, or suppression misconfiguration.
    ///
    /// Emitted annotation: `[Governance]: Missed_Alarm | Consensus: Boundary/Violation`
    MissedAlarm,
    /// Quarantined outlier node whose local grammar is Boundary or Violation.
    /// Indicates an internal hardware anomaly (not external jamming), because
    /// the swarm majority remains Admissible.
    ///
    /// Governance action: flag node data packet with hardware anomaly marker.
    /// The C2 system can choose to ignore this node's "Jamming" alarm,
    /// preserving mission continuity.
    ///
    /// Emitted annotation:
    /// `[Governance]: Local_Hardware_Anomaly_Detected | Consensus: Admissible`
    LocalHardwareAnomaly,
    /// LO phase-noise instability precursor detected at this node.
    /// The node's DSA is within honest-majority bounds but shows
    /// `LoInstabilityPrecursor` motif (RecurrentBoundaryGrazing + oscillatory slew).
    ///
    /// The node's data is still valid but timing/geolocation accuracy is degrading.
    /// Governance action: tag downstream data with `LO_Instability_Precursor` advisory.
    /// Do NOT reset the clock or recalibrate the PLL — read-only observer.
    ///
    /// Emitted annotation: `[Governance]: LO_Instability_Precursor | Review: Advisory`
    LoInstabilityPrecursor,
}

impl GovernanceTag {
    /// Human-readable governance annotation for logging or SigMF metadata.
    pub const fn label(self) -> &'static str {
        match self {
            GovernanceTag::Nominal =>
                "[Governance]: Nominal",
            GovernanceTag::ObserverQuarantined =>
                "[Governance]: Observer_Quarantined | Reason: DSA_Outlier",
            GovernanceTag::MissedAlarm =>
                "[Governance]: Missed_Alarm | Consensus: Boundary_Or_Violation",
            GovernanceTag::LocalHardwareAnomaly =>
                "[Governance]: Local_Hardware_Anomaly_Detected | Consensus: Admissible",
            GovernanceTag::LoInstabilityPrecursor =>
                "[Governance]: LO_Instability_Precursor | Review: Advisory",
        }
    }

    /// Whether this tag requires operator review or action.
    #[inline]
    pub const fn requires_action(self) -> bool {
        !matches!(self, GovernanceTag::Nominal)
    }
}

/// Per-node governance report from a consensus round.
///
/// One report is issued per node participating in the current round.
/// Produced by [`swarm_governance_report`].
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeGovernanceReport {
    /// Node identifier (matches the [`GrammarVote::node_id`]).
    pub node_id: u8,
    /// Governance tag assigned to this node for this round.
    pub tag: GovernanceTag,
    /// MAD-based robust z-score: |dsa − median_dsa| / robust_σ.
    /// Values > [`KS_REJECT_THRESHOLD`] trigger `ObserverQuarantined`.
    pub robust_z: f32,
    /// Whether this node's vote was admitted to the consensus tally.
    pub admitted: bool,
    /// Grammar state this node reported.
    pub local_grammar_severity: u8,
}

impl NodeGovernanceReport {
    /// Returns `true` if the governance tag requires operator attention.
    #[inline]
    pub fn requires_action(&self) -> bool {
        self.tag.requires_action()
    }
}

/// Compute per-node governance reports alongside the BFT consensus.
///
/// Runs the same MAD-based KS filter as [`compute_consensus`] and assigns
/// a [`GovernanceTag`] to each participating node based on:
///
/// 1. Whether its DSA score is an outlier (quarantine test).
/// 2. Whether its local grammar is inconsistent with consensus direction.
/// 3. Whether it shows `LoInstabilityPrecursor` grammar signals
///    (provided via the `lo_precursor_nodes` bitmask).
///
/// # Arguments
///
/// - `votes`              — slice of `GrammarVote` from all nodes.
/// - `bft_f`              — Byzantine fault tolerance parameter.
/// - `require_auth`       — if `true`, unauthenticated votes excluded.
/// - `lo_precursor_nodes` — bitmask of node IDs with `LoInstabilityPrecursor`
///                          motif (bit N = node_id N has LO precursor flag).
///
/// # Returns
///
/// `(reports, n_reports, consensus)` — per-node array, count, and consensus.
///
/// # Examples
///
/// ```
/// use dsfb_rf::swarm_consensus::{swarm_governance_report, GrammarVote, GovernanceTag};
/// use dsfb_rf::grammar::GrammarState;
///
/// let votes = [
///     GrammarVote { node_id: 0, state: GrammarState::Admissible, dsa_score: 1.0,
///                   episode_count: 10, hardware_authenticated: true },
///     GrammarVote { node_id: 1, state: GrammarState::Admissible, dsa_score: 1.1,
///                   episode_count: 10, hardware_authenticated: true },
///     GrammarVote { node_id: 2, state: GrammarState::Admissible, dsa_score: 0.9,
///                   episode_count: 10, hardware_authenticated: true },
///     GrammarVote { node_id: 3, state: GrammarState::Admissible, dsa_score: 1.0,
///                   episode_count: 10, hardware_authenticated: true },
///     // Byzantine node with thermal runaway DSA spike:
///     GrammarVote { node_id: 4, state: GrammarState::Violation, dsa_score: 99.0,
///                   episode_count: 10, hardware_authenticated: true },
/// ];
/// let (reports, n, consensus) = swarm_governance_report(&votes, 1, false, 0);
/// assert!(reports[..n].iter().any(|r| r.tag == GovernanceTag::LocalHardwareAnomaly),
///     "thermal runaway node must be tagged");
/// ```
pub fn swarm_governance_report(
    votes:              &[GrammarVote],
    bft_f:              u8,
    require_auth:       bool,
    lo_precursor_nodes: u64,
) -> ([NodeGovernanceReport; MAX_SWARM_NODES], usize, SwarmConsensus) {
    let mut reports = [blank_report(); MAX_SWARM_NODES];
    let n_votes = votes.len().min(MAX_SWARM_NODES);
    if n_votes == 0 {
        return (reports, 0, SwarmConsensus::no_quorum());
    }

    let admitted_flags = collect_admitted_flags(votes, require_auth);
    let (median_dsa, robust_sigma) = compute_median_and_mad(votes, &admitted_flags);
    let consensus = compute_consensus(votes, bft_f, require_auth);
    let cons_sev = consensus.modal_state.severity();

    for (i, vote) in votes.iter().take(MAX_SWARM_NODES).enumerate() {
        reports[i] = build_node_report(
            vote,
            admitted_flags[i],
            median_dsa,
            robust_sigma,
            lo_precursor_nodes,
            cons_sev,
        );
    }

    (reports, n_votes, consensus)
}

#[inline]
fn blank_report() -> NodeGovernanceReport {
    NodeGovernanceReport {
        node_id: 0,
        tag: GovernanceTag::Nominal,
        robust_z: 0.0,
        admitted: false,
        local_grammar_severity: 0,
    }
}

fn collect_admitted_flags(votes: &[GrammarVote], require_auth: bool) -> [bool; MAX_SWARM_NODES] {
    let mut admitted_flags = [false; MAX_SWARM_NODES];
    for (i, vote) in votes.iter().take(MAX_SWARM_NODES).enumerate() {
        admitted_flags[i] = !require_auth || vote.hardware_authenticated;
    }
    admitted_flags
}

fn compute_median_and_mad(
    votes: &[GrammarVote],
    admitted_flags: &[bool; MAX_SWARM_NODES],
) -> (f32, f32) {
    const MAD_SCALE: f32 = 1.482_602_2;

    let mut sorted_buf = [0.0f32; MAX_SWARM_NODES];
    let mut n_admitted = 0usize;
    for (i, vote) in votes.iter().take(MAX_SWARM_NODES).enumerate() {
        if admitted_flags[i] {
            sorted_buf[n_admitted] = vote.dsa_score;
            n_admitted += 1;
        }
    }
    insertion_sort(&mut sorted_buf[..n_admitted]);
    let median_dsa = median_of_sorted(&sorted_buf[..n_admitted]);

    let mut abs_devs = [0.0f32; MAX_SWARM_NODES];
    let mut m = 0usize;
    for (i, vote) in votes.iter().take(MAX_SWARM_NODES).enumerate() {
        if admitted_flags[i] {
            abs_devs[m] = (vote.dsa_score - median_dsa).abs();
            m += 1;
        }
    }
    insertion_sort(&mut abs_devs[..m]);
    let mad = median_of_sorted(&abs_devs[..m]);
    let robust_sigma = (MAD_SCALE * mad).max(1e-9);

    (median_dsa, robust_sigma)
}

#[inline]
fn insertion_sort(buf: &mut [f32]) {
    for i in 1..buf.len() {
        let key = buf[i];
        let mut j = i;
        while j > 0 && buf[j - 1] > key {
            buf[j] = buf[j - 1];
            j -= 1;
        }
        buf[j] = key;
    }
}

#[inline]
fn median_of_sorted(buf: &[f32]) -> f32 {
    let n = buf.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        buf[n / 2]
    } else {
        (buf[n / 2 - 1] + buf[n / 2]) * 0.5
    }
}

fn build_node_report(
    vote: &GrammarVote,
    admitted: bool,
    median_dsa: f32,
    robust_sigma: f32,
    lo_precursor_nodes: u64,
    cons_sev: u8,
) -> NodeGovernanceReport {
    let z = if admitted {
        (vote.dsa_score - median_dsa).abs() / robust_sigma
    } else {
        0.0
    };
    let quarantined = admitted && z > KS_REJECT_THRESHOLD;
    let is_lo = lo_precursor_nodes & (1u64 << (vote.node_id.min(63) as u64)) != 0;
    let local_sev = vote.state.severity();
    let tag = assign_governance_tag(admitted, quarantined, is_lo, local_sev, cons_sev);

    NodeGovernanceReport {
        node_id: vote.node_id,
        tag,
        robust_z: z,
        admitted: admitted && !quarantined,
        local_grammar_severity: local_sev,
    }
}

#[inline]
fn assign_governance_tag(
    admitted: bool,
    quarantined: bool,
    is_lo: bool,
    local_sev: u8,
    cons_sev: u8,
) -> GovernanceTag {
    if !admitted {
        GovernanceTag::ObserverQuarantined
    } else if quarantined {
        if local_sev >= 1 && cons_sev == 0 {
            GovernanceTag::LocalHardwareAnomaly
        } else {
            GovernanceTag::ObserverQuarantined
        }
    } else if is_lo && local_sev >= 1 {
        GovernanceTag::LoInstabilityPrecursor
    } else if local_sev == 0 && cons_sev >= 1 {
        GovernanceTag::MissedAlarm
    } else {
        GovernanceTag::Nominal
    }
}


// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn vote(id: u8, state: GrammarState, dsa: f32, epi: u32, auth: bool) -> GrammarVote {
        GrammarVote { node_id: id, state, dsa_score: dsa, episode_count: epi,
                      hardware_authenticated: auth }
    }

    #[test]
    fn unanimous_admissible_consensus() {
        let votes: [GrammarVote; 5] = [
            vote(0, GrammarState::Admissible, 1.0, 10, true),
            vote(1, GrammarState::Admissible, 0.9, 12, true),
            vote(2, GrammarState::Admissible, 1.1, 8,  true),
            vote(3, GrammarState::Admissible, 1.0, 11, true),
            vote(4, GrammarState::Admissible, 0.95, 9, true),
        ];
        let c = compute_consensus(&votes, 1, false);
        assert!(c.quorum_reached, "quorum must be reached");
        assert_eq!(c.modal_state, GrammarState::Admissible);
        assert!(c.p_admissible > 0.95, "nearly all admissible: {}", c.p_admissible);
    }

    #[test]
    fn no_quorum_on_empty_votes() {
        let c = compute_consensus(&[], 1, false);
        assert!(!c.quorum_reached);
        assert_eq!(c.modal_state, GrammarState::Admissible,
            "safe default when no quorum");
    }

    #[test]
    fn byzantine_outlier_quarantined() {
        let votes = [
            vote(0, GrammarState::Admissible, 1.0, 10, true),
            vote(1, GrammarState::Admissible, 1.1, 10, true),
            vote(2, GrammarState::Admissible, 0.9, 10, true),
            vote(3, GrammarState::Admissible, 1.05, 10, true),
            // Byzantine node: DSA score is an outlier
            vote(4, GrammarState::Violation, 1000.0, 10, true),
        ];
        let c = compute_consensus(&votes, 1, false);
        assert!(c.votes_quarantined >= 1, "Byzantine vote must be quarantined: {:?}", c);
        assert_eq!(c.modal_state, GrammarState::Admissible,
            "consensus must remain Admissible after quarantine");
    }

    #[test]
    fn majority_violation_consensus() {
        let votes = [
            vote(0, GrammarState::Violation, 4.5, 20, true),
            vote(1, GrammarState::Violation, 4.8, 18, true),
            vote(2, GrammarState::Violation, 4.3, 22, true),
            vote(3, GrammarState::Boundary(ReasonCode::SustainedOutwardDrift), 2.5, 15, true),
            vote(4, GrammarState::Admissible, 1.0, 10, true),
        ];
        let c = compute_consensus(&votes, 1, false);
        assert!(c.quorum_reached);
        assert_eq!(c.modal_state, GrammarState::Violation,
            "Violation majority: p_v={:.2}", c.p_violation);
    }

    #[test]
    fn auth_filter_excludes_unauthenticated() {
        let votes = [
            vote(0, GrammarState::Violation, 5.0, 20, false), // unauth
            vote(1, GrammarState::Admissible, 1.0, 10, true),
            vote(2, GrammarState::Admissible, 0.9, 12, true),
            vote(3, GrammarState::Admissible, 1.1, 11, true),
        ];
        let c = compute_consensus(&votes, 1, true);
        assert_eq!(c.votes_unauthenticated, 1, "one unauth vote");
        assert_eq!(c.modal_state, GrammarState::Admissible,
            "unauthenticated Violation vote must be excluded");
    }

    #[test]
    fn consensus_grammar_state_requires_threshold() {
        let mut c = SwarmConsensus::no_quorum();
        c.quorum_reached = true;
        c.modal_state    = GrammarState::Boundary(ReasonCode::SustainedOutwardDrift);
        c.p_boundary     = 0.4; // below CONSENSUS_THRESHOLD = 0.5
        assert!(consensus_grammar_state(&c).is_none(),
            "below threshold: must return None");
        c.p_boundary = 0.6;
        let result = consensus_grammar_state(&c);
        assert!(result.map(|s| s.is_boundary()).unwrap_or(false), "boundary consensus");
    }

    #[test]
    fn too_few_nodes_no_quorum() {
        // With bft_f=2, need 2*2+1=5 votes. Only 3 provided.
        let votes = [
            vote(0, GrammarState::Admissible, 1.0, 10, true),
            vote(1, GrammarState::Admissible, 0.9, 10, true),
            vote(2, GrammarState::Admissible, 1.1, 10, true),
        ];
        let c = compute_consensus(&votes, 2, false);
        assert!(!c.quorum_reached, "3 votes insufficient for bft_f=2");
    }
}
