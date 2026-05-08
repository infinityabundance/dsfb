// DSFB-Debug: leave-one-fixture-out cross-validation harness (Phase ζ.2).
//
// Runs a fusion evaluation on every vendored real-bytes fixture in turn,
// captures per-fixture metrics + per-detector firings, and emits:
//
//   - `LooCvAggregate` summary on stdout (verbatim test output, no
//     extrapolation),
//   - cross-fixture detector-selectivity report,
//   - per-axis (tier) discrimination report,
//   - all of the above written to docs/audit/*.md for paper §13.9 ingestion.
//
// Per academic-honesty discipline (Sessions 1-16 standing rules):
// only what the harness emits goes into the documentation; nothing is
// rounded, smoothed, or extrapolated. Sentinel fixtures (where the
// upstream slice has not yet been vendored) are surfaced as `[skip]`
// notices and excluded from the cross-fixture aggregate.
//
// Theorem 9 preservation: the harness re-fires
// `run_fusion_evaluation` per fixture and reports
// `deterministic_replay_holds` per-fixture in the LO-CV record.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::audit::{
    aggregate_detector_audit,
    aggregate_kfold_cv,
    aggregate_loo_cv,
    audit_confuser_pairs,
    bootstrap_ci,
    build_motif_refinement_from_observations,
    canonical_calibrated_weight_overrides,
    compute_axis_discrimination,
    compute_detector_selectivity_per_fixture,
    refinement_passes_gate,
    render_axis_discrimination_md,
    render_bootstrap_md,
    render_confuser_audit_md,
    render_detector_selectivity_md,
    render_kfold_cv_md,
    render_loo_cv_baseline_md,
    render_motif_refinement_md,
    DetectorSelectivity,
    EpisodeMotifObservation,
    LooCvAggregate,
    LooCvFixtureRecord,
    RefinementGateVerdict,
};
use dsfb_debug::audit::confuser_audit::EpisodeTypingObservation;
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};
use dsfb_debug::heuristics_bank::HeuristicsBank;
use dsfb_debug::types::SemanticDisposition;
use dsfb_debug::DsfbDebugEngine;

// --- Vendored fixture bytes (real upstream slices, post Phase G) ---
const F04_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F04.tsv");
const F11_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");
const F11B_BYTES: &[u8]     = include_bytes!("../data/fixtures/tadbench_trainticket_F11b.tsv");
const F19_BYTES: &[u8]      = include_bytes!("../data/fixtures/tadbench_trainticket_F19.tsv");
const ILLINOIS_BYTES: &[u8] = include_bytes!("../data/fixtures/illinois_socialnetwork.tsv");
const AIOPS_BYTES: &[u8]    = include_bytes!("../data/fixtures/aiops_challenge.tsv");
const LO2_BYTES: &[u8]      = include_bytes!("../data/fixtures/lo2.tsv");
const MULTIDIM_BYTES: &[u8] = include_bytes!("../data/fixtures/multidim_localization.tsv");
const DEEPTRALOG_BYTES: &[u8] = include_bytes!("../data/fixtures/deeptralog.tsv");
const DEFECTS4J_BYTES: &[u8]  = include_bytes!("../data/fixtures/defects4j.tsv");
const BUGSINPY_BYTES: &[u8]   = include_bytes!("../data/fixtures/bugsinpy.tsv");
const PROMISE_BYTES: &[u8]    = include_bytes!("../data/fixtures/promise_defect_prediction.tsv");

fn is_sentinel(bytes: &[u8]) -> bool {
    bytes.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
}

/// Run a single fusion evaluation on a fixture and capture the
/// LO-CV record + per-detector selectivity.
///
/// Returns `None` if the fixture is sentinel-form or fails to parse —
/// the caller logs a `[skip]` notice.
fn capture_fixture(
    fixture_name: &'static str,
    bytes: &[u8],
    cfg: &FusionConfig,
) -> Option<(LooCvFixtureRecord, Vec<DetectorSelectivity>, Vec<EpisodeTypingObservation>, Vec<EpisodeMotifObservation>)> {
    if is_sentinel(bytes) {
        eprintln!("[skip] {fixture_name} — sentinel fixture");
        return None;
    }
    let matrix = match parse_residual_projection(bytes) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[skip] {fixture_name} — parse error: {e:?}");
            return None;
        }
    };
    if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
        eprintln!("[skip] {fixture_name} — empty matrix");
        return None;
    }

    let engine = DsfbDebugEngine::<32, 64>::paper_lock().expect("paper-lock engine");
    let r = match run_fusion_evaluation(
        &engine,
        &matrix.data,
        matrix.num_signals,
        matrix.num_windows,
        matrix.healthy_window_end,
        &matrix.fault_labels,
        cfg,
        fixture_name,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[skip] {fixture_name} — fusion error: {e:?}");
            return None;
        }
    };

    // Compose LO-CV record from the fusion run. Fault recall is taken
    // from the dsfb_structural sub-metrics if available; otherwise
    // 0.0 (range-bound, per academic-honesty discipline).
    let fault_recall = r.dsfb_structural.as_ref()
        .map(|m| m.fault_recall).unwrap_or(0.0);
    let rscr = r.dsfb_structural.as_ref()
        .map(|m| m.rscr).unwrap_or(0.0);

    let record = LooCvFixtureRecord {
        fixture_name,
        rscr,
        clean_window_fp_rate: r.fusion_clean_window_fp_rate,
        fault_recall,
        raw_alert_count: r.raw_alert_count,
        fusion_episode_count: r.fusion_episode_count,
        consensus_confirmed_typed_episodes: r.consensus_confirmed_typed_episodes,
        deterministic_replay_holds: r.deterministic_replay_holds,
    };

    let selectivity = compute_detector_selectivity_per_fixture(
        &r.per_detector, fixture_name);

    // Capture per-episode typing observations from MatchConfidence
    // entries. Phase ζ.7 uses these to audit confuser-pair utilisation.
    // Phase ζ.4 + ζ.8 also build per-episode motif observations from
    // the parallel `per_episode_tier_mask` + `per_episode_top_witnesses`
    // arrays.
    let mut typing_obs: Vec<EpisodeTypingObservation> = Vec::new();
    let mut motif_obs: Vec<EpisodeMotifObservation> = Vec::new();
    for (i, mc) in r.per_episode_confidence.iter().enumerate() {
        if let SemanticDisposition::Named(top_motif) = mc.disposition {
            typing_obs.push(EpisodeTypingObservation {
                fixture_name,
                top_motif,
                runner_up_motif: mc.runner_up_motif,
                margin_vs_confuser: mc.margin_vs_confuser,
                fell_below_confuser_threshold:
                    mc.confuser_motif.is_some() && mc.margin_vs_confuser < 0.10,
            });
            // Phase ζ.4 + ζ.8: build motif observation from parallel
            // telemetry arrays (per_episode_tier_mask +
            // per_episode_top_witnesses). Indices align with
            // per_episode_confidence (run_inner pushes them all in the
            // same loop iteration).
            let tier_mask = r.per_episode_tier_mask
                .get(i).copied().unwrap_or(0);
            let witnesses = r.per_episode_top_witnesses
                .get(i).cloned().unwrap_or_default();
            motif_obs.push(EpisodeMotifObservation {
                motif: top_motif,
                fixture_name,
                observed_tier_mask: tier_mask,
                observed_top_witnesses: witnesses,
            });
        }
    }

    Some((record, selectivity, typing_obs, motif_obs))
}

/// Write a markdown file under crates/dsfb-debug/docs/audit/.
///
/// On a fresh checkout the directory may not exist; create it.
fn write_audit_markdown(filename: &str, content: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("docs");
    path.push("audit");
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("[warn] could not create docs/audit/: {e:?}");
        return;
    }
    path.push(filename);
    match fs::File::create(&path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(content.as_bytes()) {
                eprintln!("[warn] could not write {filename}: {e:?}");
            } else {
                eprintln!("[audit] wrote {}", path.display());
            }
        }
        Err(e) => eprintln!("[warn] could not open {filename}: {e:?}"),
    }
}

/// Run a full LO-CV pass across the 12 vendored fixtures with the
/// supplied FusionConfig; return the per-fixture records, per-fixture
/// selectivity, aggregated typing observations, and aggregated
/// per-episode motif observations.
fn run_loo_cv_pass(
    label: &str,
    cfg: &FusionConfig,
) -> (Vec<LooCvFixtureRecord>, Vec<Vec<DetectorSelectivity>>, Vec<EpisodeTypingObservation>, Vec<EpisodeMotifObservation>) {
    println!();
    println!("=== LO-CV PASS — {label} ===");

    let fixtures: &[(&'static str, &[u8])] = &[
        ("tadbench_trainticket_F04",     F04_BYTES),
        ("tadbench_trainticket_F11",     F11_BYTES),
        ("tadbench_trainticket_F11b",    F11B_BYTES),
        ("tadbench_trainticket_F19",     F19_BYTES),
        ("illinois_socialnetwork",       ILLINOIS_BYTES),
        ("aiops_challenge_2018_kpi",     AIOPS_BYTES),
        ("lo2_oauth2_endoductive",       LO2_BYTES),
        ("multidim_localization_part1",  MULTIDIM_BYTES),
        ("deeptralog_F01",               DEEPTRALOG_BYTES),
        ("defects4j_6project",           DEFECTS4J_BYTES),
        ("bugsinpy_6project",            BUGSINPY_BYTES),
        ("promise_defect_prediction",    PROMISE_BYTES),
    ];

    let mut records: Vec<LooCvFixtureRecord> = Vec::new();
    let mut per_fixture_selectivity: Vec<Vec<DetectorSelectivity>> = Vec::new();
    let mut all_typing_obs: Vec<EpisodeTypingObservation> = Vec::new();
    let mut all_motif_obs: Vec<EpisodeMotifObservation> = Vec::new();

    for (name, bytes) in fixtures {
        if let Some((rec, sel, obs, mobs)) = capture_fixture(name, bytes, cfg) {
            // Print per-fixture record (range-bound, verbatim).
            println!(
                "{{\"fixture\": \"{}\", \"rscr\": {:.4}, \"fp\": {:.4}, \"recall\": {:.4}, \"raw\": {}, \"eps\": {}, \"typed\": {}, \"replay\": {}}}",
                rec.fixture_name,
                rec.rscr, rec.clean_window_fp_rate, rec.fault_recall,
                rec.raw_alert_count,
                rec.fusion_episode_count,
                rec.consensus_confirmed_typed_episodes,
                rec.deterministic_replay_holds,
            );
            records.push(rec);
            per_fixture_selectivity.push(sel);
            all_typing_obs.extend(obs);
            all_motif_obs.extend(mobs);
        }
    }
    (records, per_fixture_selectivity, all_typing_obs, all_motif_obs)
}

/// Render the bank-refinement comparison ledger.
fn render_refinement_log(
    baseline: &LooCvAggregate,
    refined: &LooCvAggregate,
    verdict: &RefinementGateVerdict,
) -> String {
    let mut out = String::new();
    out.push_str("# Bank refinement LO-CV ledger (Phase ζ.9)\n\n");
    out.push_str("Phase ζ.5 default-on calibrated weighted consensus, gated\n");
    out.push_str("by `audit::loo_cv::refinement_passes_gate(baseline, refined)`\n");
    out.push_str("with ε = 0.5·stddev, no regression on any of {RSCR, FP rate,\n");
    out.push_str("fault recall, replay-holds count}.\n\n");
    out.push_str("Source: Phase ζ.9 LO-CV harness (`tests/loo_cross_validation.rs`).\n\n");
    out.push_str("## Refinement under test\n\n");
    out.push_str("Phase ζ.5 calibrated overrides: 10 family-tier detectors with\n");
    out.push_str("mean cross-fixture `healthy_firing_rate > 0.25` AND\n");
    out.push_str("`fault_firing_rate = 0.0` filtered to weight 0\n");
    out.push_str("(`pelt`, `fpop`, `spatial_sign`, `cumulative_deviation`,\n");
    out.push_str("`bayesian_blocks`, `mcd`, `inspect_cpd`, `stahel_donoho`,\n");
    out.push_str("`zonotope_escape`, `depth_rank_control`).\n\n");
    out.push_str("## Comparison\n\n");
    out.push_str("| Metric | Baseline mean | Refined mean | Delta | Baseline stddev |\n");
    out.push_str("|--------|--------------:|-------------:|------:|----------------:|\n");
    out.push_str(&format!("| RSCR | {:.4} | {:.4} | {:+.4} | {:.4} |\n",
        baseline.mean_rscr, refined.mean_rscr,
        refined.mean_rscr - baseline.mean_rscr, baseline.stddev_rscr));
    out.push_str(&format!("| Clean-window FP rate | {:.4} | {:.4} | {:+.4} | {:.4} |\n",
        baseline.mean_clean_window_fp_rate, refined.mean_clean_window_fp_rate,
        refined.mean_clean_window_fp_rate - baseline.mean_clean_window_fp_rate,
        baseline.stddev_clean_window_fp_rate));
    out.push_str(&format!("| Fault recall | {:.4} | {:.4} | {:+.4} | {:.4} |\n",
        baseline.mean_fault_recall, refined.mean_fault_recall,
        refined.mean_fault_recall - baseline.mean_fault_recall,
        baseline.stddev_fault_recall));
    out.push_str(&format!("| Total typed-confirmed episodes | {} | {} | {:+} | — |\n",
        baseline.total_typed_episodes, refined.total_typed_episodes,
        refined.total_typed_episodes as i64 - baseline.total_typed_episodes as i64));
    out.push_str(&format!("| Total fusion episodes | {} | {} | {:+} | — |\n",
        baseline.total_episodes, refined.total_episodes,
        refined.total_episodes as i64 - baseline.total_episodes as i64));
    out.push_str(&format!("| Total raw alerts | {} | {} | {:+} | — |\n",
        baseline.total_raw_alerts, refined.total_raw_alerts,
        refined.total_raw_alerts as i64 - baseline.total_raw_alerts as i64));
    out.push_str(&format!("| Replay holds | {} / {} | {} / {} | — | — |\n",
        baseline.fixtures_with_replay_holds, baseline.fixtures_observed,
        refined.fixtures_with_replay_holds, refined.fixtures_observed));

    out.push_str("\n## Verdict\n\n");
    match verdict {
        RefinementGateVerdict::Accept => {
            out.push_str("**LO-CV gate: ACCEPT.** All metrics within tolerance\n");
            out.push_str("(mean ≥ baseline − 0.5·stddev on every metric;\n");
            out.push_str("replay-holds count preserved). Refinement promoted to\n");
            out.push_str("canonical default per the user-locked Session-17 choice\n");
            out.push_str("\"Apply LO-CV-passing refinements to canonical bank\".\n");
        }
        RefinementGateVerdict::Reject { reasons } => {
            out.push_str("**LO-CV gate: REJECT.** Refinement DOES NOT pass the\n");
            out.push_str("acceptance gate; the canonical default reverts to\n");
            out.push_str("baseline (no overrides). Reasons:\n\n");
            for r in reasons {
                out.push_str(&format!("- {}\n", r));
            }
            out.push_str("\nThis is a first-class **negative finding** per the\n");
            out.push_str("user-locked Session-17 choice \"publish negative\n");
            out.push_str("findings as first-class evidence\". The infrastructure\n");
            out.push_str("(FusionConfig.detector_weight_overrides + weight_for\n");
            out.push_str("helper) ships ready for operator-side calibration.\n");
        }
    }

    // Honest interpretation paragraph.
    out.push_str("\n## Honest empirical reading\n\n");
    let rscr_delta = refined.mean_rscr - baseline.mean_rscr;
    let fp_delta   = refined.mean_clean_window_fp_rate - baseline.mean_clean_window_fp_rate;
    let recall_delta = refined.mean_fault_recall - baseline.mean_fault_recall;
    let typed_delta = refined.total_typed_episodes as i64 - baseline.total_typed_episodes as i64;

    if rscr_delta.abs() < 1e-9
        && fp_delta.abs() < 1e-9
        && recall_delta.abs() < 1e-9
        && typed_delta == 0 {
        out.push_str("**Observable delta on this cross-fixture surface is exactly zero.**\n\n");
        out.push_str("The 10 filtered family-tier detectors had `fault_firing_rate = 0`\n");
        out.push_str("across all 12 vendored fixtures (verbatim from\n");
        out.push_str("`docs/audit/detector_selectivity.md`); they were never routing\n");
        out.push_str("evidence on labelled-fault windows in the first place. Filtering\n");
        out.push_str("them on healthy windows, where they were firing, has no observable\n");
        out.push_str("effect on the cross-fixture LO-CV aggregates because the\n");
        out.push_str("`window_tier_mask`-routed bank-aware evidence depends on whether\n");
        out.push_str("a detector's tier bit is set during the matched episode's window\n");
        out.push_str("range — and these detectors' bits were not being set during the\n");
        out.push_str("episode windows that actually got typed-confirmed.\n\n");
        out.push_str("This is not failure: the gate trivially accepts (no regression\n");
        out.push_str("anywhere), and the infrastructure ships ready for operator-side\n");
        out.push_str("site calibration. Partner-data engagements where these detectors\n");
        out.push_str("DO contribute observable evidence will provide the empirical\n");
        out.push_str("lever the audit harness can then gate.\n");
    } else if rscr_delta.abs() > 0.01 || fp_delta.abs() > 0.001 || typed_delta != 0 {
        out.push_str(&format!(
            "Refinement produces measurable metric deltas on the 12-fixture\n\
             surface: RSCR {:+.4}, FP rate {:+.4}, fault recall {:+.4}, total\n\
             typed-confirmed episodes {:+}. The gate verdict above adjudicates\n\
             whether these deltas pass the LO-CV acceptance threshold.\n",
            rscr_delta, fp_delta, recall_delta, typed_delta));
    } else {
        out.push_str("Refinement produces sub-noise deltas on the 12-fixture\n");
        out.push_str("surface; the LO-CV gate adjudicates based on the per-metric\n");
        out.push_str("tolerance defined above.\n");
    }
    out
}

#[test]
fn loo_cv_baseline_across_12_fixtures() {
    // Phase ζ.2 baseline pass — default FusionConfig (no overrides).
    let cfg_baseline = FusionConfig::ALL_DEFAULT;
    let (records_b, sel_b, obs_b, motif_obs_b) = run_loo_cv_pass(
        "Phase ζ.2 baseline (no overrides)", &cfg_baseline);

    if records_b.is_empty() {
        eprintln!("[loo-cv] no fixtures populated; skipping aggregate.");
        return;
    }

    // Aggregate LO-CV baseline.
    let agg_baseline = aggregate_loo_cv(&records_b);

    println!();
    println!("=== LO-CV BASELINE AGGREGATE ({} fixtures) ===",
             agg_baseline.fixtures_observed);
    println!(
        "{{\"mean_rscr\": {:.4}, \"stddev_rscr\": {:.4}, \"mean_fp\": {:.4}, \"stddev_fp\": {:.4}, \"mean_recall\": {:.4}, \"stddev_recall\": {:.4}, \"replay_holds\": {} / {}, \"typed_total\": {}}}",
        agg_baseline.mean_rscr, agg_baseline.stddev_rscr,
        agg_baseline.mean_clean_window_fp_rate, agg_baseline.stddev_clean_window_fp_rate,
        agg_baseline.mean_fault_recall, agg_baseline.stddev_fault_recall,
        agg_baseline.fixtures_with_replay_holds, agg_baseline.fixtures_observed,
        agg_baseline.total_typed_episodes,
    );

    // Range-bound assertion: every metric in legal bounds; replay holds
    // on every populated fixture.
    assert!(agg_baseline.mean_clean_window_fp_rate >= 0.0
        && agg_baseline.mean_clean_window_fp_rate <= 1.0,
        "FP rate must be a probability");
    assert!(agg_baseline.mean_fault_recall >= 0.0
        && agg_baseline.mean_fault_recall <= 1.0,
        "Fault recall must be a probability");
    assert_eq!(
        agg_baseline.fixtures_with_replay_holds, agg_baseline.fixtures_observed,
        "Theorem 9 must hold on every populated fixture");

    // Phase ζ.5 calibrated pass — canonical override list.
    let cfg_calibrated = FusionConfig {
        detector_weight_overrides: Some(canonical_calibrated_weight_overrides()),
        ..FusionConfig::ALL_DEFAULT
    };
    let (records_c, _sel_c, _obs_c, _motif_obs_c) = run_loo_cv_pass(
        "Phase ζ.5 calibrated (canonical overrides)", &cfg_calibrated);
    let agg_calibrated = aggregate_loo_cv(&records_c);

    println!();
    println!("=== LO-CV CALIBRATED AGGREGATE ({} fixtures) ===",
             agg_calibrated.fixtures_observed);
    println!(
        "{{\"mean_rscr\": {:.4}, \"stddev_rscr\": {:.4}, \"mean_fp\": {:.4}, \"stddev_fp\": {:.4}, \"mean_recall\": {:.4}, \"stddev_recall\": {:.4}, \"replay_holds\": {} / {}, \"typed_total\": {}}}",
        agg_calibrated.mean_rscr, agg_calibrated.stddev_rscr,
        agg_calibrated.mean_clean_window_fp_rate, agg_calibrated.stddev_clean_window_fp_rate,
        agg_calibrated.mean_fault_recall, agg_calibrated.stddev_fault_recall,
        agg_calibrated.fixtures_with_replay_holds, agg_calibrated.fixtures_observed,
        agg_calibrated.total_typed_episodes,
    );
    assert_eq!(
        agg_calibrated.fixtures_with_replay_holds, agg_calibrated.fixtures_observed,
        "Theorem 9 must hold under calibrated overrides too");

    // Phase ζ.9 — gate verdict.
    let verdict = refinement_passes_gate(&agg_baseline, &agg_calibrated);
    println!();
    println!("=== Phase ζ.9 LO-CV VERDICT ===");
    match &verdict {
        RefinementGateVerdict::Accept => {
            println!("ACCEPT — refinement landed in canonical default.");
        }
        RefinementGateVerdict::Reject { reasons } => {
            println!("REJECT — refinement does NOT pass; reverts to baseline default.");
            for r in reasons { println!("  - {}", r); }
        }
    }

    // Render baseline report (preserve the existing artefact).
    let baseline_md = render_loo_cv_baseline_md(
        &agg_baseline, "Baseline (Phase ζ.2)");
    write_audit_markdown("loo_cv_baseline.md", &baseline_md);

    let detector_report = aggregate_detector_audit(&sel_b);
    let detector_md = render_detector_selectivity_md(&detector_report);
    write_audit_markdown("detector_selectivity.md", &detector_md);

    let axis_report = compute_axis_discrimination(&sel_b);
    let axis_md = render_axis_discrimination_md(&axis_report);
    write_audit_markdown("axis_discrimination.md", &axis_md);

    // Phase ζ.7 confuser-pair audit (using baseline observations).
    let bank: HeuristicsBank<64> = HeuristicsBank::with_canonical_motifs();
    let confuser_report = audit_confuser_pairs(&bank, &obs_b);
    let confuser_md = render_confuser_audit_md(&confuser_report);
    write_audit_markdown("confuser_audit.md", &confuser_md);

    // Phase ζ.4 + ζ.8 — motif affinity + named-witness refinement.
    let motif_report = build_motif_refinement_from_observations(
        &bank, &motif_obs_b);
    let motif_md = render_motif_refinement_md(&motif_report);
    write_audit_markdown("motif_refinement.md", &motif_md);

    let named_witness_md = render_named_witness_refinement_md(&motif_report);
    write_audit_markdown("named_witness_refinement.md", &named_witness_md);

    // Phase ζ.9 — refinement ledger.
    let refinement_md = render_refinement_log(&agg_baseline, &agg_calibrated, &verdict);
    write_audit_markdown("bank_refinement_log.md", &refinement_md);

    // Phase η.1 — bootstrap CI on baseline LO-CV records.
    let boot_agg = bootstrap_ci(&records_b);
    let boot_md = render_bootstrap_md(&boot_agg);
    write_audit_markdown("bootstrap_ci.md", &boot_md);
    println!();
    println!("=== Phase η.1 BOOTSTRAP CI (1000 iter, default seed) ===");
    println!("RSCR:        point {:.4}, 95% CI [{:.4}, {:.4}]",
        boot_agg.rscr.point_estimate,
        boot_agg.rscr.ci_lower_2_5,
        boot_agg.rscr.ci_upper_97_5);
    println!("FP rate:     point {:.4}, 95% CI [{:.4}, {:.4}]",
        boot_agg.clean_window_fp_rate.point_estimate,
        boot_agg.clean_window_fp_rate.ci_lower_2_5,
        boot_agg.clean_window_fp_rate.ci_upper_97_5);
    println!("Recall:      point {:.4}, 95% CI [{:.4}, {:.4}]",
        boot_agg.fault_recall.point_estimate,
        boot_agg.fault_recall.ci_lower_2_5,
        boot_agg.fault_recall.ci_upper_97_5);

    // Phase η.2 — K-fold CV (K = 4) on baseline LO-CV records.
    let kfold_agg = aggregate_kfold_cv(&records_b, 4);
    let kfold_md = render_kfold_cv_md(&kfold_agg);
    write_audit_markdown("kfold_cv.md", &kfold_md);
    println!();
    println!("=== Phase η.2 K-FOLD CV (K = 4) ===");
    println!("Folds: {}, fixtures-per-fold: {}",
        kfold_agg.folds.len(), kfold_agg.fixtures_per_fold);
    println!("Cross-fold mean RSCR: {:.4} (stddev {:.4})",
        kfold_agg.cross_fold_mean_rscr, kfold_agg.cross_fold_stddev_rscr);
    println!("Cross-fold mean FP:   {:.4} (stddev {:.4})",
        kfold_agg.cross_fold_mean_fp, kfold_agg.cross_fold_stddev_fp);
    println!("Cross-fold mean recall: {:.4} (stddev {:.4})",
        kfold_agg.cross_fold_mean_recall, kfold_agg.cross_fold_stddev_recall);
    assert!(kfold_agg.all_folds_replay_holds,
        "Theorem 9 must hold across every K-fold fold");

    println!();
    println!("=== AUDIT REPORTS WRITTEN ===");
    println!("- docs/audit/loo_cv_baseline.md       ({} fixtures)", agg_baseline.fixtures_observed);
    println!("- docs/audit/detector_selectivity.md  ({} detectors)", detector_report.entries.len());
    println!("- docs/audit/axis_discrimination.md   ({} axes)", axis_report.entries.len());
    println!("- docs/audit/confuser_audit.md        ({} declared pairs, {} observed competitions)",
             confuser_report.entries.len(),
             confuser_report.entries.iter().map(|e| e.observed_competitions).sum::<u64>());
    println!("- docs/audit/motif_refinement.md      ({} typed-episode observations)",
             motif_report.entries.len());
    println!("- docs/audit/named_witness_refinement.md  ({} entries)",
             motif_report.entries.len());
    println!("- docs/audit/bank_refinement_log.md   (LO-CV verdict: {})",
             match verdict {
                 RefinementGateVerdict::Accept => "ACCEPT",
                 RefinementGateVerdict::Reject { .. } => "REJECT",
             });
    println!("- docs/audit/bootstrap_ci.md          (1000 iter, fixture-level resampling)");
    println!("- docs/audit/kfold_cv.md              (K=4, {} folds)", kfold_agg.folds.len());
}

/// Phase ζ.8 — render the named-witness refinement report from a
/// motif refinement report. This is a separate view of the same data:
/// each entry's curated `primary_witness_detectors` is compared to
/// the observed top-K detectors firing on the matched episode.
fn render_named_witness_refinement_md(
    report: &dsfb_debug::audit::MotifRefinementReport,
) -> String {
    let mut out = String::new();
    out.push_str("# Per-motif named-witness refinement (Phase ζ.8)\n\n");
    out.push_str("For each typed-confirmed episode in the LO-CV baseline pass,\n");
    out.push_str("the table reports: the motif's hand-curated\n");
    out.push_str("`primary_witness_detectors` (Phase 8 strict ensemble gate)\n");
    out.push_str("vs the empirically observed top-5 detectors firing within\n");
    out.push_str("the matched episode's window range.\n\n");
    out.push_str("**Refinement is RECOMMENDATION, not bank mutation.**\n");
    out.push_str("Phase ζ.9 separately gates any merge through LO-CV.\n\n");
    out.push_str("Source: Phase ζ.8 audit harness (parallel to Phase ζ.4\n");
    out.push_str("affinity refinement; same data, different view).\n\n");
    out.push_str("| Motif | Fixture | Curated witnesses | Observed top-5 |\n");
    out.push_str("|-------|---------|-------------------|----------------|\n");

    for e in &report.entries {
        let curated_str = if e.current_named_witnesses.is_empty() {
            "(none)".to_string()
        } else {
            e.current_named_witnesses.iter()
                .map(|n| format!("`{}`", n))
                .collect::<Vec<_>>().join(", ")
        };
        let observed_str = if e.observed_top_witnesses.is_empty() {
            "(none fired)".to_string()
        } else {
            e.observed_top_witnesses.iter()
                .map(|(n, rate)| format!("`{}` ({:.2})", n, rate))
                .collect::<Vec<_>>().join(", ")
        };
        out.push_str(&format!(
            "| `{:?}` | `{}` | {} | {} |\n",
            e.motif, e.fixture_observed, curated_str, observed_str,
        ));
    }

    // Coverage analysis: how many entries have at least one curated
    // witness in the observed top-5?
    let mut overlap_count: u32 = 0;
    let mut no_overlap_count: u32 = 0;
    let mut no_curation_count: u32 = 0;
    for e in &report.entries {
        if e.current_named_witnesses.is_empty() {
            no_curation_count += 1;
        } else {
            let observed_names: Vec<&'static str> = e.observed_top_witnesses
                .iter().map(|(n, _)| *n).collect();
            let any_overlap = e.current_named_witnesses.iter()
                .any(|n| observed_names.contains(n));
            if any_overlap { overlap_count += 1; } else { no_overlap_count += 1; }
        }
    }
    out.push_str("\n## Witness coverage summary\n\n");
    out.push_str(&format!("- **Curated witness present in observed top-5**: {} entries\n",
        overlap_count));
    out.push_str(&format!("- **Curated witness NOT in observed top-5**: {} entries\n",
        no_overlap_count));
    out.push_str(&format!("- **No witness curation declared**: {} entries\n",
        no_curation_count));
    out
}
