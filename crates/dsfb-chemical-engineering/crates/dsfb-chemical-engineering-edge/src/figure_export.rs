//! Figure-data exporter (figure campaign WS2): emit byte-stable JSON / DOT for the evidence objects that
//! the standard `demo` run does **not** exercise, so the figure gallery can render them from the *actual*
//! crate objects (not Python-side mock-ups).
//!
//! The `demo` command already writes the per-dataset CSVs (residual streams, detector outputs, episode
//! evidence, NE107, alarm rationalization, provenance ledger, …) that most figures read. This module fills
//! the gap: it constructs **representative instances** of the named V1 evidence objects — topology /
//! propagation / causal-non-claim / provenance graphs, the confuser docket, the regime envelope, the
//! soft-sensor witness, the sensitivity sweep and ablation court, the per-detector passport, and the
//! context-overlay / amendment-chain objects — and serialises each (the graph objects via their own
//! `to_dot()` so the figure shows the canonical rendering). It also dumps the atlas authority (detectors,
//! heuristics, fault signatures, unknown taxonomy) as JSON, since those records live in the atlas crate and
//! are not in the committed atlas TOML.
//!
//! All outputs land under `<out>/figure_data/`. Everything here is deterministic and **off the replay
//! path** — it constructs fresh evidence objects for visualisation; it does not touch the sealed pipeline.

use std::path::Path;

use dsfb_chemical_engineering_atlas as authority;

use crate::ablation::AblationCourtV1;
use crate::annotation::{
    EvidenceAmendmentChainV1, MaintenanceEvent, MaintenanceEventOverlayV1,
    OperatorAnnotationLedgerV1, RecipeTransition, RecipeTransitionGuardV1,
};
use crate::confuser::ConfuserDocketV1;
use crate::detectors::{DetectorFamily, DetectorOutput};
use crate::dsfb_core::{AdmissibilityEnvelope, ResidualTriple};
use crate::passport::ChemometricPassportV1;
use crate::propagation::{CausalNonClaimEdge, CausalNonClaimGraphV1, FaultPropagationWitnessV1};
use crate::provenance_graph::{NodeKind, ProvenanceGraphBuilder};
use crate::regime_envelope::RegimeEnvelopeV1;
use crate::softsensor::SoftSensorWitnessV1;
use crate::sweep::{SensitivitySweepReceiptV1, SweepAxis};
use crate::topology::{ResidenceTimeAlignmentV1, TopologyBuilder};

/// Write `bytes` to `<dir>/<name>`, pushing a one-line summary into `log`. Errors are surfaced in the log
/// rather than aborting the whole export (a single bad write must not lose the rest of the gallery data).
fn write(dir: &Path, name: &str, bytes: &str, log: &mut Vec<String>) {
    match std::fs::write(dir.join(name), bytes) {
        Ok(()) => log.push(format!("  figure_data/{name}  ({} bytes)", bytes.len())),
        Err(e) => log.push(format!("  figure_data/{name}  WRITE FAILED: {e}")),
    }
}

/// Serialise `value` to pretty JSON (falling back to an error string that is still valid JSON).
fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

/// Build representative instances of every not-exercised evidence object and write JSON/DOT for the figure
/// gallery. Returns a list of verbose log lines (what was written) for the captured build log.
pub fn export_all(out_root: &Path) -> Vec<String> {
    let mut log = Vec::new();
    let dir = out_root.join("figure_data");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        log.push(format!("figure_data: create_dir_all FAILED: {e}"));
        return log;
    }
    log.push(
        "evidence-object figure data (representative instances; off the replay path):".to_string(),
    );

    // ── Atlas authority dump (records live in the atlas crate, not the TOML) ────────────────────────────
    write(
        &dir,
        "atlas_detectors.json",
        &json(&authority::DETECTOR_RECORDS),
        &mut log,
    );
    write(
        &dir,
        "atlas_heuristics.json",
        &json(&authority::HEURISTIC_RECORDS),
        &mut log,
    );
    write(
        &dir,
        "atlas_fault_signatures.json",
        &json(&authority::FAULT_SIGNATURES),
        &mut log,
    );
    write(
        &dir,
        "atlas_unknown_taxonomy.json",
        &json(&authority::UNKNOWN_TAXONOMY),
        &mut log,
    );

    // ── D — graphs (topology / propagation / causal-non-claim / provenance) ─────────────────────────────
    // The P64 multi-unit demonstrator: feed → reactor → separator with declared residence times.
    let mut tb = TopologyBuilder::new("feed_reactor_separator");
    tb.unit("feed", "feed")
        .unit("reactor", "reactor")
        .unit("separator", "separator");
    tb.flow("feed", "reactor", 3.0, "min")
        .flow("reactor", "separator", 5.0, "min");
    let topo = tb.seal();
    write(&dir, "topology.dot", &topo.to_dot(), &mut log);
    write(&dir, "topology.json", &json(&topo), &mut log);

    // Residence-time alignment: downstream = upstream delayed by the declared residence (3 samples).
    let upstream: Vec<f64> = (0..80)
        .map(|i| (i as f64 * 0.25).sin() + 0.4 * (i as f64 * 0.05).cos())
        .collect();
    let mut downstream = vec![0.0; 3];
    downstream.extend(upstream.iter().take(77).copied());
    let align =
        ResidenceTimeAlignmentV1::build("reactor", "separator", &upstream, &downstream, 3.0, 1.0);
    write(&dir, "residence_alignment.json", &json(&align), &mut log);
    // Also persist the two series so the figure can plot the overlay.
    let mut series = String::from("t,upstream,downstream\n");
    for (i, (u, d)) in upstream
        .iter()
        .zip(downstream.iter().chain(std::iter::repeat(&f64::NAN)))
        .enumerate()
    {
        series.push_str(&format!(
            "{i},{u:.6},{}\n",
            if d.is_nan() {
                String::new()
            } else {
                format!("{d:.6}")
            }
        ));
    }
    write(&dir, "residence_series.csv", &series, &mut log);

    // Fault-propagation witness: reactor onset @100, separator onset @105; declared residence lag 5, tol 2.
    let prop = FaultPropagationWitnessV1::build("reactor", "separator", 100, 105, 5, 2);
    write(&dir, "propagation_witness.json", &json(&prop), &mut log);

    // Causal-non-claim graph: precedence + topology edges, rendered with the sealed NO-CAUSAL-CLAIM disclaimer.
    let edges = vec![
        CausalNonClaimEdge {
            from: "feed".into(),
            to: "reactor".into(),
            precedence_lag: 3,
            topologically_upstream: true,
        },
        CausalNonClaimEdge {
            from: "reactor".into(),
            to: "separator".into(),
            precedence_lag: 5,
            topologically_upstream: true,
        },
    ];
    let cnc = CausalNonClaimGraphV1::build("feed_reactor_separator", edges);
    write(&dir, "causal_non_claim.dot", &cnc.to_dot(), &mut log);

    // Residual provenance graph: raw → residual → detector → episode → label → court_root.
    let mut pb = ProvenanceGraphBuilder::new("cstr_reactor");
    pb.node("raw:T_reactor", NodeKind::Raw, "reactor temperature");
    pb.node("res:T_reactor", NodeKind::Residual, "T_reactor − baseline");
    pb.node("det:ewma_spe", NodeKind::Detector, "EWMA(SPE)");
    pb.node("det:pca_t2", NodeKind::Detector, "PCA T²");
    pb.node("ep:0", NodeKind::Episode, "episode #0 (EnvViolation)");
    pb.node(
        "lbl:H3",
        NodeKind::Label,
        "reactor thermal excursion candidate",
    );
    pb.node("court:root", NodeKind::CourtRoot, "evidence_root abc123…");
    pb.link("raw:T_reactor", "res:T_reactor");
    pb.link("res:T_reactor", "det:ewma_spe");
    pb.link("res:T_reactor", "det:pca_t2");
    pb.link("det:ewma_spe", "ep:0");
    pb.link("det:pca_t2", "ep:0");
    pb.link("ep:0", "lbl:H3");
    pb.link("lbl:H3", "court:root");
    let prov = pb.seal();
    write(&dir, "provenance.dot", &prov.to_dot(), &mut log);
    write(&dir, "provenance.json", &prov.to_json(), &mut log);

    // ── E — confuser docket (cites the catalogued confusers of the first executed fault signature) ──────
    let fid = authority::FAULT_SIGNATURES[0].fault_id;
    if let Some(docket) = ConfuserDocketV1::for_fault("idx=120..168", fid) {
        write(&dir, "confuser_docket.json", &json(&docket), &mut log);
    }

    // ── G — soft-sensor witness (debutanizer C4 content; deterministic-envelope soft sensor) ────────────
    let measured: Vec<f64> = (0..60)
        .map(|i| 0.30 + 0.05 * (i as f64 * 0.2).sin())
        .collect();
    let prediction: Vec<f64> = (0..60)
        .map(|i| 0.30 + 0.045 * (i as f64 * 0.2).sin() + 0.01)
        .collect();
    let interval: Vec<f64> = vec![0.02; 60];
    let ss = SoftSensorWitnessV1::build(
        "C4_content",
        "deterministic-envelope",
        "debutanizer baseline rows 0..500; 7 cheap inputs",
        &measured,
        &prediction,
        &interval,
    );
    write(&dir, "softsensor_witness.json", &json(&ss), &mut log);

    // ── H/regime — a calibrated regime envelope (penicillin growth phase) ───────────────────────────────
    let default_env = AdmissibilityEnvelope::symmetric(3.0, 0.1);
    let baseline: Vec<ResidualTriple> = (0..96)
        .map(|i| ResidualTriple {
            r: (i % 7) as f64 * 0.2,
            delta: 0.1,
            sigma: 0.2,
            timestamp: i as f64,
        })
        .collect();
    let regime = RegimeEnvelopeV1::calibrate_and_seal(
        "penicillin:growth",
        "growth",
        "reactor",
        "process_structure",
        &baseline,
        default_env,
        16,
        0.99,
    );
    write(&dir, "regime_envelope.json", &json(&regime), &mut log);

    // ── I60 — sensitivity sweep + ablation court ────────────────────────────────────────────────────────
    // Sweep the quorum thresholds (k detectors × min families); metric = a toy robustness score (episode
    // count proxy) that is monotone in the thresholds, so the heatmap shows a smooth robustness surface.
    let axes = vec![
        SweepAxis {
            name: "quorum_k".into(),
            values: vec![2.0, 3.0, 4.0, 5.0],
        },
        SweepAxis {
            name: "min_families".into(),
            values: vec![1.0, 2.0, 3.0],
        },
    ];
    let sweep = SensitivitySweepReceiptV1::run("episode_count_proxy", axes, |c| {
        30.0 - 3.0 * c[0] - 2.0 * c[1]
    });
    write(&dir, "sweep_receipt.json", &json(&sweep), &mut log);

    // Ablation: full pipeline metric vs ablating each DSFB component (drift / slew / envelope / family-quorum).
    let ablation = AblationCourtV1::run(
        "fused_episodes",
        || 6.0,
        vec![
            ("drift".to_string(), (|| 2.0) as fn() -> f64),
            ("slew".to_string(), (|| 5.0) as fn() -> f64),
            (
                "admissibility_envelope".to_string(),
                (|| 3.0) as fn() -> f64,
            ),
            ("family_quorum".to_string(), (|| 4.0) as fn() -> f64),
        ],
    );
    write(&dir, "ablation_court.json", &json(&ablation), &mut log);

    // ── I59 — per-detector chemometric passport ─────────────────────────────────────────────────────────
    let pbaseline = vec![
        vec![0.0, 1.0],
        vec![0.1, 0.9],
        vec![-0.1, 1.1],
        vec![0.05, 1.02],
    ];
    let pinput = vec![vec![0.0, 1.0], vec![2.0, 3.0], vec![2.4, 3.1]];
    let pouts: Vec<DetectorOutput> = (0..3)
        .map(|i| DetectorOutput {
            detector_id: "pca_t2".into(),
            family: DetectorFamily::ClassicalMspc,
            time_index: i,
            variable_scope: "scores".into(),
            raw_score: i as f64 + 1.0,
            normalized_score: i as f64 + 1.0,
            threshold: 1.0,
            signed_margin: i as f64,
            breach: i > 0,
        })
        .collect();
    let passport = ChemometricPassportV1::build(
        "pca_t2",
        "ClassicalMspc",
        &pbaseline,
        &pinput,
        &pouts,
        "baseline 99th-percentile",
        "z-score(baseline mean/std)",
        "non-finite->NaN, counted",
    );
    write(&dir, "passport.json", &json(&passport), &mut log);

    // ── I57/I58 — context overlays + the human-review chains ────────────────────────────────────────────
    let guard = RecipeTransitionGuardV1::build(
        "penicillin_fedbatch",
        vec![RecipeTransition {
            at_index: 600,
            from_phase: "growth".into(),
            to_phase: "production".into(),
        }],
    );
    write(&dir, "recipe_guard.json", &json(&guard), &mut log);
    let maint = MaintenanceEventOverlayV1::build(
        "swat",
        vec![MaintenanceEvent {
            start_index: 100,
            end_index: 150,
            description: "pump P-101 service".into(),
        }],
    );
    write(&dir, "maintenance_overlay.json", &json(&maint), &mut log);

    let mut ledger = OperatorAnnotationLedgerV1::new("cstr_reactor");
    ledger.append(
        "2026-05-25T10:00:00Z",
        "idx=120..168",
        "operator_A",
        "checked TC-3, looks like drift",
    );
    ledger.append(
        "2026-05-25T10:05:00Z",
        "idx=120..168",
        "operator_B",
        "scheduled recalibration; opened MOC-2231",
    );
    write(&dir, "annotation_ledger.json", &json(&ledger), &mut log);

    let mut chain = EvidenceAmendmentChainV1::new("e".repeat(64));
    chain.amend(
        "2026-05-25T11:00:00Z",
        "clarification",
        "onset re-dated to sample 162 after operator review",
    );
    chain.amend(
        "2026-05-25T11:30:00Z",
        "correction",
        "TC-3 confirmed biased; episode reclassified SENSOR_QUALITY",
    );
    write(&dir, "amendment_chain.json", &json(&chain), &mut log);

    log.push(format!(
        "evidence-object figure data: {} files written to {}",
        log.len() - 1,
        dir.display()
    ));
    log
}
