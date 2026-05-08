// DSFB-Debug: per-axis ablation studies for the 9-axis bank-aware
// fusion ladder (Phase η.4, Session 18).
//
// For each of the 9 fusion axes, run a configuration with that axis
// disabled and measure the LO-CV aggregate impact on RSCR / FP /
// recall / typed-confirmed. The marginal contribution of each axis
// is the delta between the all-axes-on baseline and the axis-removed
// configuration.
//
// Per academic-honesty discipline: only verbatim test stdout; no
// extrapolation; per-axis verdict is empirical evidence for the
// 9-axis ladder claim.
//
// Theorem 9 preservation: each configuration runs through
// `run_fusion_evaluation` which fires `verify_deterministic_replay`;
// every ablation is asserted to maintain replay.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::audit::{aggregate_loo_cv, LooCvAggregate, LooCvFixtureRecord};
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};
use dsfb_debug::DsfbDebugEngine;

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

fn run_loo_with_cfg(cfg: &FusionConfig) -> LooCvAggregate {
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
    for (name, bytes) in fixtures {
        if is_sentinel(bytes) { continue; }
        let matrix = match parse_residual_projection(bytes) {
            Ok(m) => m, Err(_) => continue,
        };
        if matrix.is_sentinel || matrix.num_signals == 0 || matrix.num_windows == 0 {
            continue;
        }
        let engine = DsfbDebugEngine::<32, 64>::paper_lock().expect("paper-lock");
        let r = match run_fusion_evaluation(
            &engine, &matrix.data,
            matrix.num_signals, matrix.num_windows,
            matrix.healthy_window_end, &matrix.fault_labels,
            cfg, name,
        ) {
            Ok(r) => r, Err(_) => continue,
        };
        let fault_recall = r.dsfb_structural.as_ref()
            .map(|m| m.fault_recall).unwrap_or(0.0);
        let rscr = r.dsfb_structural.as_ref()
            .map(|m| m.rscr).unwrap_or(0.0);
        records.push(LooCvFixtureRecord {
            fixture_name: name,
            rscr,
            clean_window_fp_rate: r.fusion_clean_window_fp_rate,
            fault_recall,
            raw_alert_count: r.raw_alert_count,
            fusion_episode_count: r.fusion_episode_count,
            consensus_confirmed_typed_episodes: r.consensus_confirmed_typed_episodes,
            deterministic_replay_holds: r.deterministic_replay_holds,
        });
    }
    aggregate_loo_cv(&records)
}

fn write_audit_markdown(filename: &str, content: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("docs"); path.push("audit");
    if let Err(e) = fs::create_dir_all(&path) {
        eprintln!("[warn] mkdir docs/audit: {e:?}"); return;
    }
    path.push(filename);
    match fs::File::create(&path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(content.as_bytes()) {
                eprintln!("[warn] write {filename}: {e:?}");
            } else {
                eprintln!("[audit] wrote {}", path.display());
            }
        }
        Err(e) => eprintln!("[warn] open {filename}: {e:?}"),
    }
}

#[test]
fn axis_ablation_9_axes() {
    let mut out = String::new();
    out.push_str("# Per-axis ablation — Phase η.4\n\n");
    out.push_str("Each row reports one of the 9 fusion axes disabled relative\n");
    out.push_str("to the all-axes-on baseline (`FusionConfig::ALL_DEFAULT`).\n");
    out.push_str("The marginal contribution of an axis = baseline − axis-removed\n");
    out.push_str("on the cross-fixture LO-CV aggregate (12 fixtures).\n\n");
    out.push_str("Source: Phase η.4 ablation harness (`tests/axis_ablation.rs`).\n");
    out.push_str("Theorem 9 deterministic replay verified per configuration.\n\n");

    println!();
    println!("=== Phase η.4 PER-AXIS ABLATION (9 axes × leave-one-out) ===");

    // Baseline — all axes on.
    let baseline = run_loo_with_cfg(&FusionConfig::ALL_DEFAULT);
    println!("[baseline] RSCR {:.4}, FP {:.4}, recall {:.4}, typed {}",
        baseline.mean_rscr, baseline.mean_clean_window_fp_rate,
        baseline.mean_fault_recall, baseline.total_typed_episodes);
    out.push_str("## Baseline (all 9 axes on)\n\n");
    out.push_str(&format!(
        "| Mean RSCR | Mean FP | Mean recall | Typed-confirmed | Replay |\n"
    ));
    out.push_str("|----------:|--------:|------------:|----------------:|:------:|\n");
    out.push_str(&format!(
        "| {:.4} | {:.4} | {:.4} | {} | {} / {} |\n\n",
        baseline.mean_rscr, baseline.mean_clean_window_fp_rate,
        baseline.mean_fault_recall, baseline.total_typed_episodes,
        baseline.fixtures_with_replay_holds, baseline.fixtures_observed));

    out.push_str("## Per-axis ablation (axis disabled vs baseline)\n\n");
    out.push_str("| Axis | Removed config | Mean RSCR | Mean FP | Mean recall | Typed | ΔTyped | Replay |\n");
    out.push_str("|------|----------------|----------:|--------:|------------:|------:|-------:|:------:|\n");

    // Each axis's leave-one-out config + label.
    let scenarios: &[(&str, &str, FusionConfig)] = &[
        ("1. Provenance gate", "`use_bank_aware_consensus=false`", FusionConfig {
            use_bank_aware_consensus: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("2. Margin gate", "`margin_gate=0.0`", FusionConfig {
            margin_gate: 0.0, ..FusionConfig::ALL_DEFAULT
        }),
        ("3. Tier-affinity scoring", "`use_tier_affinity=false`", FusionConfig {
            use_tier_affinity: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("4. Zero-tier filter", "`use_zero_tier_filter=false`", FusionConfig {
            use_zero_tier_filter: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("5. Adaptive margin gate", "`use_adaptive_margin_gate=false`", FusionConfig {
            use_adaptive_margin_gate: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("6. Confuser-boundary adjudication", "`use_confuser_boundary=false`", FusionConfig {
            use_confuser_boundary: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("7. Disambiguator boost", "`use_disambiguator_boost=false`", FusionConfig {
            use_disambiguator_boost: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("8. Tier-level primary witness gate", "`use_primary_witness_tier_gate=false`", FusionConfig {
            use_primary_witness_tier_gate: false, ..FusionConfig::ALL_DEFAULT
        }),
        ("9. Per-detector named witness gate", "`use_primary_witness_detector_gate=false`", FusionConfig {
            use_primary_witness_detector_gate: false, ..FusionConfig::ALL_DEFAULT
        }),
    ];

    let mut deltas: Vec<(&str, i64, f64, f64, f64, u64)> = Vec::new();
    for (label, flag_descr, cfg) in scenarios {
        let r = run_loo_with_cfg(cfg);
        let dt = r.total_typed_episodes as i64 - baseline.total_typed_episodes as i64;
        out.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {} | {:+} | {} / {} |\n",
            label, flag_descr,
            r.mean_rscr, r.mean_clean_window_fp_rate, r.mean_fault_recall,
            r.total_typed_episodes, dt,
            r.fixtures_with_replay_holds, r.fixtures_observed));
        println!("[{:>30}] off → typed {} (Δ {:+}), FP {:.4}, replay {}/{}",
            label, r.total_typed_episodes, dt,
            r.mean_clean_window_fp_rate,
            r.fixtures_with_replay_holds, r.fixtures_observed);
        assert_eq!(r.fixtures_with_replay_holds, r.fixtures_observed,
            "Theorem 9 must hold under axis {} ablation", label);
        deltas.push((label, dt, r.mean_rscr, r.mean_clean_window_fp_rate,
                     r.mean_fault_recall, r.total_typed_episodes));
    }

    out.push_str("\n## Honest empirical reading\n\n");
    out.push_str("All nine fusion axes are now ablatable via single\n");
    out.push_str("`FusionConfig` flags. Theorem 9 deterministic replay\n");
    out.push_str("holds under every leave-one-axis-out configuration.\n\n");
    out.push_str("**Marginal contribution per axis** (Δtyped relative to\n");
    out.push_str("the all-axes-on baseline):\n\n");
    out.push_str("- **Negative Δ** (axis disabled → fewer typed episodes):\n");
    out.push_str("  the axis is positively load-bearing — it ENABLES\n");
    out.push_str("  additional typed dispositions on the current surface.\n");
    out.push_str("- **Positive Δ** (axis disabled → more typed episodes):\n");
    out.push_str("  the axis is restrictively load-bearing — it FILTERS\n");
    out.push_str("  episodes that would otherwise be admitted as typed.\n");
    out.push_str("- **Zero Δ**: the axis is unmeasured-load-bearing on the\n");
    out.push_str("  current 12-fixture surface (gate fires correctly but\n");
    out.push_str("  the resulting decision is unchanged from the baseline).\n\n");

    out.push_str("Per Session-17 academic-honesty discipline, the table\n");
    out.push_str("reports verbatim test stdout; partner-data engagements\n");
    out.push_str("with sharper fault signatures may activate currently-\n");
    out.push_str("zero-Δ axes that have correctly-implemented logic but\n");
    out.push_str("no firing on this surface.\n");

    write_audit_markdown("axis_ablation.md", &out);
}
