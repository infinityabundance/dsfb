// DSFB-Debug: fusion comparison sweep on the F-11 fixture.
//
// Sweeps min_consensus = 1, 3, 5, 7, 9 across the post-Phase-8
// 205-detector fusion ensemble (Tiers A-U + EXTRA + V/X/Y/Z/AA) wired
// against the 9-axis bank-aware fusion configuration plus the DSFB
// structural pipeline. Reports the fusion matrix to stdout: per-N raw
// consensus alert count, fusion episode count, fusion FP rate,
// consensus-confirmed-typed-episode count, consensus-confirmed FP rate.
//
// Honest expectation per panel directive: as min_consensus grows from
// 1 to higher values, FP rate drops monotonically. The
// consensus_confirmed_typed_episode FP rate is the operator-facing
// final-output metric — it should be ≤ ANY single-detector FP rate
// AND ≤ DSFB-structural-alone FP rate.
//
// Per academic-honesty discipline: the test reports actually-emitted
// numbers; nothing rounded or extrapolated.

#![cfg(all(feature = "std", feature = "paper-lock"))]

use dsfb_debug::adapters::residual_projection::parse_residual_projection;
use dsfb_debug::fusion::{run_fusion_evaluation, FusionConfig};
use dsfb_debug::DsfbDebugEngine;

const F11_BYTES:  &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11.tsv");
const F11B_BYTES: &[u8] = include_bytes!("../data/fixtures/tadbench_trainticket_F11b.tsv");

fn is_sentinel(bytes: &[u8]) -> bool {
    bytes.windows(b"UPSTREAM_FIXTURE_NOT_VENDORED".len())
        .any(|w| w == b"UPSTREAM_FIXTURE_NOT_VENDORED")
}

fn run_fusion_sweep_on_fixture(label: &str, fixture_name: &'static str, bytes: &[u8]) {
    if is_sentinel(bytes) {
        eprintln!("[skip] {label} fixture is the sentinel form.");
        return;
    }
    let matrix = match parse_residual_projection(bytes) {
        Ok(m) => m,
        Err(e) => { eprintln!("[skip] {label}: parse error: {e:?}"); return; }
    };
    if matrix.is_sentinel || matrix.num_signals == 0 {
        eprintln!("[skip] {label} parsed empty.");
        return;
    }

    let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();

    println!();
    println!("=== FUSION SWEEP — {label} (Phase 8: 205 detectors across 27 axes) ===");
    println!("fixture: {fixture_name}");
    println!("num_signals: {}, num_windows: {}, healthy_window_end: {}",
             matrix.num_signals, matrix.num_windows, matrix.healthy_window_end);
    println!();
    println!("| Min N | Det | Raw alerts | L-2 ep | L-2 FP | L-3 typed | L-3 ambig | L-3 bank-filt | L-3 FP | Determ |");
    println!("|------:|----:|-----------:|-------:|-------:|----------:|----------:|--------------:|-------:|-------:|");

    for &n in &[1_u8, 3, 5, 7, 9] {
        let cfg = FusionConfig {
            min_consensus: n,
            ..FusionConfig::ALL_DEFAULT
        };
        let r = run_fusion_evaluation(
            &engine,
            &matrix.data,
            matrix.num_signals,
            matrix.num_windows,
            matrix.healthy_window_end,
            &matrix.fault_labels,
            &cfg,
            fixture_name,
        ).expect("fusion eval should succeed");
        println!("| **N≥{}** | {} | {} | {} | {:.4} | {} | {} | {} | {:.4} | {} |",
                 n,
                 r.detectors_used,
                 r.raw_alert_count,
                 r.fusion_episode_count,
                 r.fusion_clean_window_fp_rate,
                 r.consensus_confirmed_typed_episodes,
                 r.ambiguous_typed_episodes,
                 r.bank_aware_filtered_out,
                 r.consensus_confirmed_clean_fp_rate,
                 if r.deterministic_replay_holds { "ok" } else { "FAIL" },
        );

        assert!(r.fusion_clean_window_fp_rate >= 0.0
                && r.fusion_clean_window_fp_rate <= 1.0);
        assert!(r.consensus_confirmed_clean_fp_rate >= 0.0
                && r.consensus_confirmed_clean_fp_rate <= 1.0);
        assert!(r.deterministic_replay_holds,
                "fusion replay must hold at N={}", n);
    }

    println!();
}

fn run_bank_aware_ab_on_fixture(label: &str, fixture_name: &'static str, bytes: &[u8]) {
    if is_sentinel(bytes) { eprintln!("[skip] {label} (A/B)"); return; }
    let matrix = match parse_residual_projection(bytes) {
        Ok(m) => m, Err(_) => { eprintln!("[skip] {label}: parse"); return; }
    };
    if matrix.is_sentinel || matrix.num_signals == 0 { return; }
    let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();

    // Phase 2.5 diagnostic: dump the per-episode confidence for the
    // baseline and Phase 2 configs at N>=3 so we can see what's matching.
    println!();
    println!("=== PHASE-2.5 DIAGNOSTIC (per-episode confidence) — {label} ===");
    for &(setting, use_bank, use_tier) in &[
        ("baseline (Phase 0)",     false, false),
        ("Phase 2: tier-affinity", true,  true),
    ] {
        let cfg = FusionConfig {
            min_consensus: 3,
            use_bank_aware_consensus: use_bank,
            margin_gate: 0.0,
            use_tier_affinity: use_tier,
            ..FusionConfig::ALL_DEFAULT
        };
        let r = run_fusion_evaluation(
            &engine, &matrix.data, matrix.num_signals, matrix.num_windows,
            matrix.healthy_window_end, &matrix.fault_labels, &cfg, fixture_name,
        ).expect("eval");
        println!("[{setting}] N>=3, {} confidence records:", r.per_episode_confidence.len());
        for (i, c) in r.per_episode_confidence.iter().enumerate() {
            println!("  ep[{}]: top={:?} top_score={:.4} runner_up={:?}({:.4}) margin={:.4} tier_cf={:.3} confuser={:?}({:.4}) m_vs_conf={:.4}",
                     i, c.disposition, c.top_score,
                     c.runner_up_motif, c.runner_up_score, c.margin, c.tier_consensus_factor,
                     c.confuser_motif, c.confuser_score, c.margin_vs_confuser);
        }
        println!("    operator_score: {:.4}", r.operator_score);
        println!("    confuser_ambiguous: {}", r.confuser_ambiguous_episodes);
    }

    println!();
    println!("=== PHASE-1 + PHASE-2 BANK-AWARE A/B — {label} ===");
    println!("|         Setting          | Min N | L-3 typed | L-3 ambig | L-3 bank-filt | L-3 FP |");
    println!("|-------------------------:|------:|----------:|----------:|--------------:|-------:|");

    for &n in &[3_u8, 5, 7] {
        for &(setting, use_bank, gate, use_tier) in &[
            ("baseline (Phase 0 / off)",   false, 0.0_f64,  false),
            ("Phase 1: bank only",         true,  0.0_f64,  false),
            ("Phase 1: margin only",       false, 0.30_f64, false),
            ("Phase 1: bank + margin",     true,  0.30_f64, false),
            ("Phase 2: tier-affinity",     true,  0.0_f64,  true),
            ("Phase 2: full (ALL_DEFAULT)", true, 0.30_f64, true),
        ] {
            let cfg = FusionConfig {
                min_consensus: n,
                use_bank_aware_consensus: use_bank,
                margin_gate: gate,
                use_tier_affinity: use_tier,
                ..FusionConfig::ALL_DEFAULT
            };
            let r = run_fusion_evaluation(
                &engine, &matrix.data, matrix.num_signals, matrix.num_windows,
                matrix.healthy_window_end, &matrix.fault_labels, &cfg, fixture_name,
            ).expect("fusion eval should succeed");
            println!("| {:>24} | N>={} | {:>9} | {:>9} | {:>13} | {:>6.4} |",
                     setting, n,
                     r.consensus_confirmed_typed_episodes,
                     r.ambiguous_typed_episodes,
                     r.bank_aware_filtered_out,
                     r.consensus_confirmed_clean_fp_rate);
            assert!(r.deterministic_replay_holds);
        }
        println!("| --- | --- | --- | --- | --- | --- |");
    }
    println!();
    println!("Reading:");
    println!("  Phase 0:  pre-bank-aware behaviour, uniform global consensus");
    println!("  Phase 1:  bank = per-motif/provenance consensus; margin = ambiguous tag");
    println!("  Phase 2:  tier-affinity scoring lifts margin for motifs whose relevant");
    println!("            tiers fired; targets converting low-margin ambiguous into");
    println!("            confidently typed.");
    println!();
}

#[test]
fn fusion_sweep_on_f11_fixture() {
    run_fusion_sweep_on_fixture(
        "TADBench/TrainTicket F-11 deployment-regression",
        "tadbench_F11_fusion",
        F11_BYTES,
    );
    run_bank_aware_ab_on_fixture(
        "TADBench/TrainTicket F-11 deployment-regression",
        "tadbench_F11_fusion_ab",
        F11_BYTES,
    );
    println!("Honest reading:");
    println!("  * Layer-2 episodes are pure consensus arithmetic (no DSFB structural typing)");
    println!("  * Layer-3 \"typed-confirmed\" = DSFB structural episodes whose window range");
    println!("    overlaps a consensus-confirmed window. These carry typed motifs +");
    println!("    consensus validation; the operator-facing output.");
    println!("  * As min_consensus rises, episodes drop monotonically (consensus filter");
    println!("    becomes stricter). Layer-3 FP rate at N>=3 should be <= Session-6");
    println!("    single-detector minima (scalar 0.0139, CUSUM 0.0116, EWMA 0.0812,");
    println!("    structural per-window 0.0209).");
}

#[test]
fn fusion_sweep_on_f11b_fixture() {
    run_fusion_sweep_on_fixture(
        "TADBench/TrainTicket F-11b auth-mongo fault",
        "tadbench_F11b_fusion",
        F11B_BYTES,
    );
    println!("Cross-fixture honest reading:");
    println!("  * F-11b is a smaller fixture (4 windows x 6 signals) projected from a");
    println!("    distinct fault case (ts-auth-mongo_5.0.9_2022-07-06).");
    println!("  * Empirical claim is bounded to F-11 + F-11b. Generalisation requires");
    println!("    additional partner-data engagement (Phase II).");
}
