//! T.12.g — Graph / Topology Anomaly: the seventh real
//! literature expansion proposal filed through the T.12.0
//! amendment court.
//!
//! **Panel-locked commit identity**:
//!
//! > **T.12.g files the Graph / Topology Anomaly amendment
//! > proposal. It admits only deterministic topology witnesses
//! > whose graph model, baseline, update law, metric definition,
//! > normalization, node / edge identity law, and decision
//! > functional are declared; resolves SEED collisions;
//! > classifies metric variants as parameterizations; rejects
//! > community / embedding / random-walk claims unless
//! > deterministically reduced; and preserves the frozen T.10
//! > corpus hash.**
//!
//! **Main panel warning (panel-locked)**: *"A graph metric is
//! not a detector until the baseline, update law, metric law,
//! and decision law are declared."*
//!
//! ## Method: SEED collision walk BEFORE canonical assignment
//!
//! T.12.g's design began with a grep of [`crate::seed::SEED`]
//! for every graph / topology candidate. The walk found
//! **one** graph-adjacent primitive already canonical:
//!
//! * **Fanout cascade** at SEED id 43 — debug-trace cascade
//!   primitive. Recognised here as the shared cascade ancestor
//!   for the `GraphAnomalyDetection` source class.
//!
//! The corpus is graph-anomaly-sparse: most graph / topology
//! anomaly primitives have no SEED ancestor. T.12.g is
//! therefore CanonicalAddition-heavy. Eight genuinely new
//! canonicals at reserved ids 5701..=5708 are admitted with
//! declared graph-model + baseline + update-law + metric-law +
//! decision-law contracts:
//!
//! * **Degree spike** (5701) — node-degree time-series
//!   exceedance against a declared baseline. Required graph
//!   contract: graph type (directed / undirected / bipartite /
//!   multigraph), node identity law, edge identity law, edge
//!   weight law, baseline (graph or window), threshold,
//!   tie-break.
//! * **Betweenness shift** (5702) — betweenness-centrality
//!   temporal shift. Required graph contract: shortest-path
//!   law (Dijkstra / Brandes / Floyd-Warshall), weighted /
//!   unweighted, directed / undirected, normalization,
//!   tie-handling, baseline window, threshold.
//! * **Clustering-coefficient shift** (5703) — local clustering
//!   coefficient temporal shift. Required graph contract:
//!   triangle-count law, normalization (transitivity vs local-
//!   clustering vs average), baseline window, threshold.
//! * **PageRank residual** (5704) — PageRank temporal shift.
//!   Required graph contract: directed / undirected, edge-
//!   weight law, damping factor d (declared, typical 0.85),
//!   iteration count OR convergence threshold, dangling-node
//!   handling (uniform-distribute / self-loop / sink),
//!   normalization, baseline comparison.
//! * **Edge-cut anomaly** (5705) — min-cut / spectral-cut
//!   shift. Required graph contract: cut algorithm
//!   (Stoer-Wagner / Karger deterministic / spectral),
//!   weighted / unweighted, baseline cut value, threshold.
//! * **Bridge-node emergence** (5706) — articulation-point
//!   emergence. Required graph contract: connectivity
//!   definition (vertex connectivity / edge connectivity),
//!   DFS-based detection law (Tarjan articulation), baseline,
//!   threshold.
//! * **Cascade precursor** (5707) — temporal cascade
//!   predictor. Required graph contract: temporal-edge-
//!   ordering law, causal-adjacency approximation, hop limit,
//!   fanout threshold, minimum cycle support, clock-skew
//!   confuser handling. Structurally distinct from SEED 43
//!   Fanout cascade (which detects an active fanout; cascade
//!   precursor PREDICTS one before it fully unfolds).
//! * **Motif-count anomaly** (5708) — count of a declared
//!   motif. Required graph contract: motif definition (node /
//!   edge count + topology), node / edge labels included or
//!   excluded, directed / undirected, count normalization,
//!   baseline comparison.
//!
//! Three parameterizations:
//!
//! * **Weighted-degree spike** (5709) →
//!   `ParameterizationOf(Degree spike, 5701)` — adds edge-
//!   weight summation.
//! * **k-hop fanout** (5710) → `ParameterizationOf(Fanout
//!   cascade, SEED 43)` — adds declared k-hop neighbourhood
//!   law.
//! * **Directed motif-count** (5711) →
//!   `ParameterizationOf(Motif-count anomaly, 5708)` — adds
//!   directed-edge-aware motif enumeration.
//!
//! Two rejections (T.12.g is the **first proposal** to
//! exercise two `RejectedNotDeterministic` records in one
//! commit):
//!
//! * **Community boundary shift** (5712) —
//!   `RejectedNotDeterministic`. Community detection algorithms
//!   (Louvain, label propagation, Leiden, Infomap) are
//!   randomized / implementation-sensitive in origin. Admission
//!   requires algorithm + sample seed + tie-break + modularity
//!   rule + resolution parameter + convergence law declared.
//! * **Random-walk embedding anomaly** (5713) —
//!   `RejectedNotDeterministic`. DeepWalk / node2vec / random-
//!   walk-based embedding shifts are randomized in origin.
//!   Admission requires walk seed + walk length + walk count +
//!   tie-break + embedding-projection matrix anchor + numeric
//!   mode declared.
//!
//! ## Court-delta categories the proposal exercises
//!
//! All five panel-locked court-delta categories (the wire-name
//! set closed at T.12.d remains closed):
//!
//! * `CanonicalAddition` ×8 — degree spike, betweenness shift,
//!   clustering-coef shift, PageRank residual, edge-cut
//!   anomaly, bridge-node emergence, cascade precursor, motif-
//!   count anomaly.
//! * `ExistingCanonicalAuthorityResolution` ×1 — Fanout cascade
//!   (SEED 43).
//! * `DomainTransferOf` ×1 — Fanout cascade (SEED 43) as
//!   shared cascade ancestor for `GraphAnomalyDetection`.
//! * `ParameterizationOf` ×3 — weighted-degree spike,
//!   k-hop fanout, directed motif-count.
//! * `RejectedNotDeterministic` ×2 — community boundary shift,
//!   random-walk embedding anomaly. **First proposal to
//!   exercise TWO rejection records in one commit.**
//!
//! Total: 8 + 1 + 1 + 3 + 2 = **15 dedup-court records**.
//!
//! ## Hash posture (panel-locked, MUST hold)
//!
//! * `corpus_hash_v1` byte-identical (no SEED mutation).
//! * `SEED.len()` stays at 54.
//! * `corpus_hash_v2` NOT created.
//! * Every prior T.11/S1.3/T.12.x hash and every
//!   `DetectorPassport` hash byte-identical.
//! * R.12b episodes 13/89/1917 byte-stable.
//! * **NEW**: a non-trivial T.12.g graph
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

/// Reserved canonical id for Degree spike. 5701..5713 is the
/// T.12.g bucket.
pub const DEGREE_SPIKE_RESERVED_CANONICAL_ID: u32 = 5701;

/// Reserved canonical id for Betweenness shift.
pub const BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID: u32 = 5702;

/// Reserved canonical id for Clustering-coefficient shift.
pub const CLUSTERING_SHIFT_RESERVED_CANONICAL_ID: u32 = 5703;

/// Reserved canonical id for PageRank residual.
pub const PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID: u32 = 5704;

/// Reserved canonical id for Edge-cut anomaly.
pub const EDGE_CUT_RESERVED_CANONICAL_ID: u32 = 5705;

/// Reserved canonical id for Bridge-node emergence
/// (articulation-point emergence).
pub const BRIDGE_NODE_RESERVED_CANONICAL_ID: u32 = 5706;

/// Reserved canonical id for Cascade precursor (temporal
/// cascade predictor; structurally distinct from SEED 43
/// Fanout cascade which detects an active fanout).
pub const CASCADE_PRECURSOR_RESERVED_CANONICAL_ID: u32 = 5707;

/// Reserved canonical id for Motif-count anomaly.
pub const MOTIF_COUNT_RESERVED_CANONICAL_ID: u32 = 5708;

/// Reserved id for Weighted-degree spike.
/// `ParameterizationOf(Degree spike, 5701)`.
pub const WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID: u32 = 5709;

/// Reserved id for k-hop fanout.
/// `ParameterizationOf(Fanout cascade, SEED 43)`.
pub const K_HOP_FANOUT_RESERVED_PRIMITIVE_ID: u32 = 5710;

/// Reserved id for Directed motif-count.
/// `ParameterizationOf(Motif-count anomaly, 5708)`.
pub const DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID: u32 = 5711;

/// Reserved id for Community boundary shift.
/// `RejectedNotDeterministic`. Community detection algorithms
/// (Louvain, Leiden, label propagation, Infomap) are
/// randomized / implementation-sensitive in origin. Admission
/// requires algorithm + sample seed + tie-break + modularity
/// rule + resolution parameter + convergence law declared.
pub const COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID: u32 = 5712;

/// Reserved id for Random-walk embedding anomaly
/// (DeepWalk / node2vec / random-walk-based embeddings).
/// `RejectedNotDeterministic`. Random walks are randomized in
/// origin. Admission requires walk seed + walk length + walk
/// count + tie-break + embedding-projection matrix anchor +
/// numeric mode declared.
pub const RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID: u32 = 5713;

// Existing SEED canonical id referenced by T.12.g.

/// Fanout cascade --- SEED canonical id 43 (debug-trace
/// cascade primitive). Shared cascade ancestor for the
/// GraphAnomalyDetection source class.
pub const FANOUT_CASCADE_SEED_ID: u32 = 43;

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
// Builders for the graph expansion batch
// ---------------------------------------------------------------

/// Build the graph `CorpusExpansionBatch` body.
fn build_graph_expansion_batch() -> crate::amendment::CorpusExpansionBatch {
    build_expansion_batch(
        "t12_g_graph_first_proposal",
        SourceClass::GraphAnomalyDetection,
        graph_proposed_primitives(),
        graph_proposed_aliases(),
        graph_proposed_dedup_records(),
        graph_proposed_genealogy_edges(),
        graph_proposed_source_refs(),
    )
}

/// Thirteen proposed primitives: 8 canonical + 3 parameterization
/// + 2 rejection shells.
fn graph_proposed_primitives() -> Vec<ProposedPrimitive> {
    vec![
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(DEGREE_SPIKE_RESERVED_CANONICAL_ID),
            display_name: "Degree spike",
            motivation: "Node-degree time-series exceedance detector. Required graph \
                 contract: graph type (directed / undirected / bipartite / multigraph), \
                 node identity law, edge identity law, edge weight law (treated as \
                 unweighted unless WeightedDegreeSpike parameterization invoked), \
                 baseline (graph snapshot or sliding window), threshold, tie-break, \
                 confuser notes (graph-size change, ID churn). Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID),
            display_name: "Betweenness shift",
            motivation: "Betweenness-centrality temporal shift detector. Required \
                 graph contract: shortest-path law (Dijkstra for weighted / Brandes \
                 for unweighted / Floyd-Warshall for small all-pairs), weighted vs \
                 unweighted, directed vs undirected, normalization (n-2 choose 2 or \
                 (n-1)(n-2)/2), tie-handling for equal-shortest-paths, baseline \
                 window, threshold. Deterministic when shortest-path law is fixed.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CLUSTERING_SHIFT_RESERVED_CANONICAL_ID),
            display_name: "Clustering-coefficient shift",
            motivation: "Local clustering coefficient temporal shift. Required graph \
                 contract: triangle-count law (open vs closed triplets), normalization \
                 (transitivity vs local-clustering vs average-clustering), baseline \
                 window, threshold. Deterministic.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID),
            display_name: "PageRank residual",
            motivation: "PageRank temporal shift detector (Brin & Page 1998). \
                 Required graph contract: directed vs undirected, edge weight law, \
                 damping factor d (declared, typical 0.85), iteration count OR \
                 convergence threshold, dangling-node handling (uniform-distribute / \
                 self-loop / sink), normalization (L1-sum-to-1), baseline \
                 comparison law. Deterministic when damping + iteration / convergence \
                 + dangling-node handling are pinned.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(EDGE_CUT_RESERVED_CANONICAL_ID),
            display_name: "Edge-cut anomaly",
            motivation: "Min-cut / spectral-cut shift detector. Required graph \
                 contract: cut algorithm (deterministic Stoer-Wagner / Karger \
                 deterministic variant / spectral Fiedler-vector), weighted vs \
                 unweighted, baseline cut value, threshold. Deterministic when \
                 cut algorithm is pinned.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(BRIDGE_NODE_RESERVED_CANONICAL_ID),
            display_name: "Bridge-node emergence (articulation point)",
            motivation: "Articulation-point emergence detector. Required graph \
                 contract: connectivity definition (vertex connectivity / edge \
                 connectivity), DFS-based articulation detection law (Tarjan 1972), \
                 baseline, threshold for emergence. Deterministic given DFS traversal \
                 order (tie-break law: ascending node id).",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(CASCADE_PRECURSOR_RESERVED_CANONICAL_ID),
            display_name: "Cascade precursor (temporal predictor)",
            motivation: "Temporal cascade predictor: predicts a fanout cascade \
                 BEFORE it fully unfolds based on temporal-edge-ordering signature. \
                 Structurally distinct from SEED 43 Fanout cascade (which detects \
                 an active fanout). Required graph contract: temporal-edge-ordering \
                 law (event-time monotonicity), causal-adjacency approximation \
                 (Lamport / vector-clock / wall-time), hop limit, fanout threshold \
                 for prediction, minimum cycle support, clock-skew confuser \
                 handling. Deterministic given fixed event-time anchoring.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(MOTIF_COUNT_RESERVED_CANONICAL_ID),
            display_name: "Motif-count anomaly",
            motivation: "Count of a declared motif pattern (Milo et al. 2002 network \
                 motifs). Required graph contract: motif definition (node count + \
                 edge count + topology pattern), node / edge labels included or \
                 excluded, directed vs undirected (DirectedMotifCount \
                 parameterization invoked for the directed case), enumeration law \
                 (subgraph enumeration order), count normalization (raw count vs \
                 z-score against random-graph null), baseline comparison law. \
                 Deterministic when enumeration order is pinned.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID),
            display_name: "Weighted-degree spike - parameterization shell",
            motivation: "Edge-weight-summing parameterization of Degree spike (5701). \
                 The court rules: weighted-degree variant is ParameterizationOf \
                 (Degree spike, 5701), NOT a new canonical primitive. Appears in \
                 proposed_primitives but NOT in new_canonical_records.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            display_name: "k-hop fanout - parameterization shell",
            motivation: "k-hop neighbourhood parameterization of Fanout cascade (SEED \
                 id 43). The court rules: k-hop fanout is ParameterizationOf \
                 (Fanout cascade, SEED 43) with declared k (typical k=2 or k=3), \
                 NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID),
            display_name: "Directed motif-count - parameterization shell",
            motivation: "Directed-edge-aware parameterization of Motif-count anomaly \
                 (5708). The court rules: directed motif-count is ParameterizationOf \
                 (Motif-count anomaly, 5708) with declared edge-direction-aware \
                 enumeration, NOT a new canonical primitive.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID),
            display_name: "Community boundary shift - rejected shell",
            motivation: "Community detection algorithms (Louvain Blondel 2008, Leiden \
                 Traag 2019, label propagation, Infomap) are randomized / \
                 implementation-sensitive in origin: tie-break in modularity-gain \
                 sorting depends on traversal order, and several algorithms include \
                 random restarts. The court does NOT admit community boundary shift \
                 to the dedup-court delta's new_canonical_records. A future T.12.x \
                 may admit a Deterministic_Community_Partition_Proxy canonical only \
                 if the algorithm choice, sample seed, tie-break rule, modularity \
                 rule, resolution parameter, and convergence law are all brutally \
                 explicit.",
        },
        ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID),
            display_name: "Random-walk embedding anomaly - rejected shell",
            motivation: "DeepWalk (Perozzi et al. 2014) / node2vec (Grover & Leskovec \
                 2016) and the random-walk-based graph embedding family are \
                 randomized in origin: each invocation samples fresh random walks \
                 over the graph. The court does NOT admit random-walk embedding \
                 anomaly to the dedup-court delta's new_canonical_records. A future \
                 T.12.x may admit a Deterministic_Walk_Embedding_Proxy canonical \
                 only if the walk seed, walk length, walk count, tie-break for \
                 equal-probability neighbours, embedding-projection matrix anchor \
                 (closed-form OR pinned-fixture-hash), and numeric mode are all \
                 brutally explicit.",
        },
    ]
}

/// Zero alias claims.
fn graph_proposed_aliases() -> Vec<ProposedAliasClaim> {
    Vec::new()
}

/// Fifteen dedup-court decisions on the graph batch.
fn graph_proposed_dedup_records() -> Vec<ProposedDedupRecord> {
    vec![
        // -- 8 CanonicalAddition records ---------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(DEGREE_SPIKE_RESERVED_CANONICAL_ID),
            reason: "Degree spike: node-degree exceedance. Declared graph + decision \
                 law: graph type (directed / undirected / bipartite / multigraph), \
                 node identity law, edge identity law, edge weight law, baseline \
                 (graph snapshot or sliding window), threshold, tie-break, confuser \
                 notes (graph-size change, ingestion gaps, ID churn).",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Betweenness shift: betweenness-centrality temporal shift. \
                 Declared graph + decision law: shortest-path law (Dijkstra / \
                 Brandes / Floyd-Warshall), weighted vs unweighted, directed vs \
                 undirected, normalization, tie-handling for equal-shortest-paths, \
                 baseline window, threshold.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CLUSTERING_SHIFT_RESERVED_CANONICAL_ID),
            reason: "Clustering-coefficient shift: local clustering coefficient \
                 temporal shift. Declared graph + decision law: triangle-count law, \
                 normalization (transitivity vs local-clustering vs average-\
                 clustering), baseline window, threshold.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID),
            reason: "PageRank residual (Brin & Page 1998): PageRank temporal shift. \
                 Declared graph + decision law: directed vs undirected, edge weight \
                 law, damping factor d, iteration count OR convergence threshold, \
                 dangling-node handling (uniform-distribute / self-loop / sink), \
                 normalization, baseline comparison.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(EDGE_CUT_RESERVED_CANONICAL_ID),
            reason: "Edge-cut anomaly: min-cut / spectral-cut shift. Declared graph \
                 + decision law: cut algorithm (Stoer-Wagner / deterministic Karger \
                 / spectral Fiedler-vector), weighted vs unweighted, baseline cut \
                 value, threshold.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(BRIDGE_NODE_RESERVED_CANONICAL_ID),
            reason: "Bridge-node emergence (articulation point): Tarjan 1972 DFS-\
                 based articulation detection. Declared graph + decision law: \
                 connectivity definition (vertex connectivity / edge connectivity), \
                 DFS-based articulation detection law, tie-break (ascending node \
                 id), baseline, threshold for emergence.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(CASCADE_PRECURSOR_RESERVED_CANONICAL_ID),
            reason: "Cascade precursor (temporal predictor): predicts a fanout \
                 cascade BEFORE it fully unfolds. Structurally distinct from SEED \
                 43 Fanout cascade (which detects an active fanout). Declared graph \
                 + decision law: temporal-edge-ordering law (event-time monotonicity), \
                 causal-adjacency approximation, hop limit, fanout threshold, \
                 minimum cycle support, clock-skew confuser handling.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_CANONICAL_ADDITION,
            canonical_id: DetectorCanonicalId(MOTIF_COUNT_RESERVED_CANONICAL_ID),
            reason: "Motif-count anomaly (Milo et al. 2002): count of declared motif. \
                 Declared graph + decision law: motif definition (node count + edge \
                 count + topology pattern), node / edge labels included or excluded, \
                 directed vs undirected, motif-enumeration law (subgraph enumeration \
                 order), count normalization (raw count vs z-score against random-\
                 graph null), baseline comparison law.",
        },
        // -- 1 ExistingCanonicalAuthorityResolution record ----
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
            canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            reason: "Fanout cascade stays canonical at SEED id 43. Cross-class \
                 adjacency: the GraphAnomalyDetection source class recognises Fanout \
                 cascade as the shared cascade ancestor. Declared graph + decision \
                 law: fanout structure detection over event-graph trace, threshold, \
                 sampling law. No duplicate admitted; k-hop fanout (record 5710 \
                 below) collapses here as ParameterizationOf.",
        },
        // -- 1 DomainTransferOf record -----------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_DOMAIN_TRANSFER_OF,
            canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            reason: "Fanout cascade (SEED id 43) is recognised by the \
                 GraphAnomalyDetection source class as the shared cascade ancestor. \
                 The court records the domain transfer without re-canonicalising \
                 Fanout cascade. The graph-specific cascade predictor (5707, Cascade \
                 precursor) is structurally distinct and admitted as a separate \
                 canonical.",
        },
        // -- 3 ParameterizationOf records --------------------
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID),
            reason: "Weighted-degree spike is ParameterizationOf(Degree spike, 5701). \
                 The edge-weight summation is the parameterization; the family-level \
                 decision functional is Degree spike's. The court declines to admit \
                 weighted-degree spike as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            reason: "k-hop fanout is ParameterizationOf(Fanout cascade, SEED id 43). \
                 The k-hop neighbourhood law is the parameterization; the family-\
                 level decision functional is Fanout cascade's. The court declines \
                 to admit k-hop fanout as a new canonical primitive.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_PARAMETERIZATION_OF,
            canonical_id: DetectorCanonicalId(DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID),
            reason: "Directed motif-count is ParameterizationOf(Motif-count anomaly, \
                 5708). The directed-edge-aware enumeration is the parameterization; \
                 the family-level decision functional is Motif-count anomaly's. The \
                 court declines to admit directed motif-count as a new canonical \
                 primitive.",
        },
        // -- 2 RejectedNotDeterministic records --------------
        // T.12.g is the first proposal to exercise TWO
        // RejectedNotDeterministic records in one commit.
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID),
            reason: "Community boundary shift (Louvain Blondel 2008, Leiden Traag \
                 2019, label propagation, Infomap) is randomized / implementation-\
                 sensitive in origin. Tie-break in modularity-gain sorting depends \
                 on traversal order; several algorithms include random restarts. \
                 Rejected as a literature-original canonical primitive for this \
                 deterministic corpus unless reduced to a declared deterministic \
                 proxy (algorithm choice, sample seed, tie-break rule, modularity \
                 rule, resolution parameter, and convergence law all brutally \
                 explicit) in a later T.12.x proposal. Deterministic stance: the \
                 rejection is on the randomization alone; the modularity / cut-\
                 boundary functional is deterministic given a fixed partition.",
        },
        ProposedDedupRecord {
            decision_wire_name: CATEGORY_REJECTED_NOT_DETERMINISTIC,
            canonical_id: DetectorCanonicalId(RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID),
            reason: "Random-walk embedding anomaly (DeepWalk Perozzi 2014; node2vec \
                 Grover & Leskovec 2016) is randomized in origin: each invocation \
                 samples fresh random walks over the graph. Rejected as a \
                 literature-original canonical primitive for this deterministic \
                 corpus unless reduced to a declared deterministic proxy (walk seed, \
                 walk length, walk count, tie-break for equal-probability \
                 neighbours, embedding-projection matrix anchor closed-form OR \
                 pinned-fixture-hash, and numeric mode all brutally explicit) in a \
                 later T.12.x proposal.",
        },
    ]
}

/// Four genealogy edges proposed for the post-freeze graph.
/// The corpus is graph-anomaly-sparse, so most new canonicals
/// have no SEED ancestor; only Cascade precursor descends from
/// Fanout cascade. The three parameterizations carry
/// ParameterVariantOf edges.
fn graph_proposed_genealogy_edges() -> Vec<ProposedGenealogyEdge> {
    vec![
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(CASCADE_PRECURSOR_RESERVED_CANONICAL_ID),
            to_canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            edge_kind_wire_name: "DerivedFrom",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(DEGREE_SPIKE_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(K_HOP_FANOUT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(FANOUT_CASCADE_SEED_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
        ProposedGenealogyEdge {
            from_canonical_id: DetectorCanonicalId(DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID),
            to_canonical_id: DetectorCanonicalId(MOTIF_COUNT_RESERVED_CANONICAL_ID),
            edge_kind_wire_name: "ParameterVariantOf",
        },
    ]
}

/// Ten source refs supporting the graph expansion.
fn graph_proposed_source_refs() -> Vec<ProposedSourceRef> {
    vec![
        ProposedSourceRef {
            citation_key: "newman_networks_2010",
            title: "Networks: An Introduction",
            year: 2010,
            venue: "Oxford University Press (degree distribution chapter)",
        },
        ProposedSourceRef {
            citation_key: "freeman_betweenness_1977",
            title: "A Set of Measures of Centrality Based on Betweenness",
            year: 1977,
            venue: "Sociometry 40(1)",
        },
        ProposedSourceRef {
            citation_key: "watts_strogatz_clustering_1998",
            title: "Collective Dynamics of Small-World Networks",
            year: 1998,
            venue: "Nature 393 (clustering-coefficient origin)",
        },
        ProposedSourceRef {
            citation_key: "brin_page_pagerank_1998",
            title: "The Anatomy of a Large-Scale Hypertextual Web Search Engine",
            year: 1998,
            venue: "Computer Networks and ISDN Systems 30(1-7) (PageRank)",
        },
        ProposedSourceRef {
            citation_key: "stoer_wagner_mincut_1997",
            title: "A Simple Min-Cut Algorithm",
            year: 1997,
            venue: "Journal of the ACM 44(4) (deterministic global min-cut)",
        },
        ProposedSourceRef {
            citation_key: "tarjan_articulation_1972",
            title: "Depth-First Search and Linear Graph Algorithms",
            year: 1972,
            venue: "SIAM Journal on Computing 1(2) (articulation-point DFS)",
        },
        ProposedSourceRef {
            citation_key: "milo_motif_2002",
            title: "Network Motifs: Simple Building Blocks of Complex Networks",
            year: 2002,
            venue: "Science 298(5594)",
        },
        ProposedSourceRef {
            citation_key: "iribarren_moro_cascade_2009",
            title: "Impact of Human Activity Patterns on the Dynamics of Information Diffusion",
            year: 2009,
            venue: "Physical Review Letters 103(3) (temporal cascade precursor)",
        },
        ProposedSourceRef {
            citation_key: "blondel_louvain_2008",
            title: "Fast Unfolding of Communities in Large Networks (rejection-shell reference; community detection is randomized / implementation-sensitive)",
            year: 2008,
            venue: "Journal of Statistical Mechanics 2008(10)",
        },
        ProposedSourceRef {
            citation_key: "perozzi_deepwalk_2014",
            title: "DeepWalk: Online Learning of Social Representations (rejection-shell reference; random-walk embeddings are randomized)",
            year: 2014,
            venue: "KDD 2014",
        },
    ]
}

/// Build the graph `DedupCourtDelta`.
fn build_graph_dedup_delta() -> crate::amendment::DedupCourtDelta {
    build_dedup_court_delta(
        "t12_g_graph_delta",
        vec![
            DetectorCanonicalId(DEGREE_SPIKE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CLUSTERING_SHIFT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(EDGE_CUT_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(BRIDGE_NODE_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(CASCADE_PRECURSOR_RESERVED_CANONICAL_ID),
            DetectorCanonicalId(MOTIF_COUNT_RESERVED_CANONICAL_ID),
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

/// Build the T.12.g graph `CorpusAmendmentProposal`. Two builds
/// against this static seed produce byte-identical bytes.
#[must_use]
pub fn seed_t12_g_graph_proposal() -> CorpusAmendmentProposal {
    build_amendment_proposal(
        "t12_g_graph_first_proposal",
        "T.12.g files the Graph / Topology Anomaly amendment proposal. Adds eight \
         genuinely new canonical graph primitives (degree spike, betweenness shift, \
         clustering-coefficient shift, PageRank residual, edge-cut anomaly, bridge-\
         node emergence, cascade precursor, motif-count anomaly) at reserved \
         canonical ids 5701..=5708 with declared graph-model + baseline + update-law \
         + metric-law + decision-law contracts (graph type + node / edge identity + \
         edge weight + shortest-path law + damping + iteration / convergence + \
         dangling-node handling + cut algorithm + connectivity definition + \
         temporal-edge-ordering + motif enumeration + normalization + threshold). \
         Records one ExistingCanonicalAuthorityResolution decision keeping Fanout \
         cascade (SEED id 43) canonical under the GraphAnomalyDetection source class \
         without duplication. Records one DomainTransferOf decision naming Fanout \
         cascade as the shared cascade ancestor. Records three ParameterizationOf \
         decisions: weighted-degree spike is ParameterizationOf(Degree spike); \
         k-hop fanout is ParameterizationOf(Fanout cascade); directed motif-count \
         is ParameterizationOf(Motif-count anomaly). Rejects TWO graph-anomaly \
         literature records as RejectedNotDeterministic (the FIRST T.12.x proposal \
         with two rejection records in one commit): community boundary shift (5712 \
         - Louvain / Leiden / label propagation / Infomap are randomized / \
         implementation-sensitive; admission requires algorithm + seed + tie-break \
         + modularity rule + resolution parameter + convergence law declared) and \
         random-walk embedding anomaly (5713 - DeepWalk / node2vec are randomized; \
         admission requires walk seed + walk length + walk count + tie-break + \
         embedding-projection matrix anchor + numeric mode declared). Every record's \
         reason text declares its specific graph-model + decision-law contract - \
         the panel-locked warning was 'a graph metric is not a detector until the \
         baseline, update law, metric law, and decision law are declared'. Does \
         NOT mutate SEED (SEED.len() stays at 54); status = Open pending review.",
        SourceClass::GraphAnomalyDetection,
        build_graph_expansion_batch(),
        build_graph_dedup_delta(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_g_graph",
    )
}
