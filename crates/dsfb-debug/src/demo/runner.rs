//! Per-fixture pipeline driver.
//!
//! Reads each vendored fixture from `data/fixtures/`, runs the engine
//! end-to-end, captures per-(window, signal) `SignalEvaluation` data,
//! closed `DebugEpisode` records, fusion `per_detector` outputs, and
//! the JSON metric block. Hands everything to the figure-rendering
//! and PDF-assembly modules.

#![cfg(feature = "demo")]

use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;
use std::{format, vec};

use crate::adapters::residual_projection::{parse_residual_projection, OwnedResidualMatrix};
use crate::error::Result;
use crate::fusion::{run_fusion_evaluation, FusionConfig, FusionMetrics};
use crate::real_data::{
    evaluate_real_dataset,
    RealDatasetEvaluation,
    RealDatasetManifest,
    MANIFEST_AIOPS_CHALLENGE,
    MANIFEST_BUGSINPY,
    MANIFEST_DEEPTRALOG,
    MANIFEST_DEFECTS4J,
    MANIFEST_ILLINOIS_SOCIALNETWORK,
    MANIFEST_LO2,
    MANIFEST_MULTIDIM_LOCALIZATION,
    MANIFEST_PROMISE,
    MANIFEST_TADBENCH_F04,
    MANIFEST_TADBENCH_F11,
    MANIFEST_TADBENCH_F11B,
    MANIFEST_TADBENCH_F19,
};
use crate::types::*;
use crate::DsfbDebugEngine;

use crate::demo::figures;
use crate::demo::infrastructure;
use crate::demo::pdf_report;

/// Per-fixture run result. All per-(w,s) data captured for figure rendering.
pub struct FixtureRun {
    pub manifest_name: String,
    pub matrix: OwnedResidualMatrix,
    pub eval_grid: Vec<SignalEvaluation>, // num_windows * num_signals
    pub episodes: Vec<DebugEpisode>,
    pub real_data_eval: RealDatasetEvaluation,
    pub fusion_metrics: FusionMetrics,
}

/// Top-level entry point invoked by `src/bin/dsfb_debug_demo.rs`.
///
/// Steps:
///   1. Make `output-dsfb-debug/dsfb-debug-{datetime}/` folder tree.
///   2. Render architecture + infrastructure figures.
///   3. For each of 12 vendored fixtures: run engine + render figures + write JSON.
///   4. Render cross-fixture summary figures.
///   5. Assemble PDF report.
///   6. Zip artefacts into sibling `dsfb-debug-{datetime}.zip`.
///   7. Print absolute path of the zip to stdout.
pub fn run_demo() -> Result<PathBuf> {
    let datetime = format_utc_timestamp();
    let run_id = format!("dsfb-debug-{}", datetime);
    let output_root = PathBuf::from("output-dsfb-debug").join(&run_id);
    create_dir_all(&output_root).map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;

    let figs_root = output_root.join("figures");
    let results_root = output_root.join("results");
    let cross_root = figs_root.join("cross_fixture");
    let arch_root = figs_root.join("00_architecture");
    create_dir_all(&figs_root).ok();
    create_dir_all(&results_root).ok();
    create_dir_all(&cross_root).ok();
    create_dir_all(&arch_root).ok();

    std::println!("[dsfb-debug-demo] Run id: {}", run_id);
    std::println!("[dsfb-debug-demo] Output: {}", output_root.display());

    // -- Architecture / infrastructure figures (3) -----------------------
    std::println!("[dsfb-debug-demo] Rendering architecture figures...");
    infrastructure::render_pipeline_figure(&arch_root.join("01_pipeline.png"))?;
    infrastructure::render_tier_breakdown(&arch_root.join("02_tier_breakdown.png"))?;
    infrastructure::render_motif_affinity_heatmap(&arch_root.join("03_motif_affinity.png"))?;

    // -- Bank metadata (motif x affinity, used by Python figure pipeline) -
    let bank_path = results_root.join("bank.json");
    write(&bank_path, serialize_bank_json()).ok();

    // -- Per-fixture pipeline (12 fixtures × 10 figures) ----------------
    let fixtures: Vec<(&'static str, &'static [u8], &'static RealDatasetManifest)> = std::vec![
        ("01_F11", include_bytes!("../../data/fixtures/tadbench_trainticket_F11.tsv"),     &MANIFEST_TADBENCH_F11),
        ("02_F11b", include_bytes!("../../data/fixtures/tadbench_trainticket_F11b.tsv"),   &MANIFEST_TADBENCH_F11B),
        ("03_F04", include_bytes!("../../data/fixtures/tadbench_trainticket_F04.tsv"),     &MANIFEST_TADBENCH_F04),
        ("04_F19", include_bytes!("../../data/fixtures/tadbench_trainticket_F19.tsv"),     &MANIFEST_TADBENCH_F19),
        ("05_Illinois", include_bytes!("../../data/fixtures/illinois_socialnetwork.tsv"),  &MANIFEST_ILLINOIS_SOCIALNETWORK),
        ("06_AIOpsChallenge", include_bytes!("../../data/fixtures/aiops_challenge.tsv"),   &MANIFEST_AIOPS_CHALLENGE),
        ("07_LO2", include_bytes!("../../data/fixtures/lo2.tsv"),                          &MANIFEST_LO2),
        ("08_MultiDim", include_bytes!("../../data/fixtures/multidim_localization.tsv"),   &MANIFEST_MULTIDIM_LOCALIZATION),
        ("09_DeepTraLog", include_bytes!("../../data/fixtures/deeptralog.tsv"),            &MANIFEST_DEEPTRALOG),
        ("10_Defects4J", include_bytes!("../../data/fixtures/defects4j.tsv"),              &MANIFEST_DEFECTS4J),
        ("11_BugsInPy", include_bytes!("../../data/fixtures/bugsinpy.tsv"),                &MANIFEST_BUGSINPY),
        ("12_PROMISE", include_bytes!("../../data/fixtures/promise_defect_prediction.tsv"),&MANIFEST_PROMISE),
    ];

    let mut all_runs: Vec<FixtureRun> = Vec::new();

    for (label, bytes, manifest) in &fixtures {
        std::println!("[dsfb-debug-demo] {label}: parsing fixture...");
        let fixture_dir = figs_root.join(label);
        create_dir_all(&fixture_dir).ok();
        match run_one_fixture(label, bytes, manifest) {
            Ok(run) => {
                figures::render_per_fixture(&run, &fixture_dir)?;
                let json = serialize_metrics_json(&run);
                let json_path = results_root.join(format!("{}.json", label));
                write(&json_path, json).ok();
                all_runs.push(run);
            }
            Err(e) => {
                std::eprintln!("[dsfb-debug-demo] {label}: skipped ({:?})", e);
            }
        }
    }

    // -- Cross-fixture summary figures (3) -------------------------------
    std::println!("[dsfb-debug-demo] Rendering cross-fixture summary figures...");
    figures::render_rscr_scatter(&all_runs, &cross_root.join("01_rscr_scatter.png"))?;
    figures::render_fusion_sweep_curve(&all_runs, &cross_root.join("02_fusion_sweep.png"))?;
    figures::render_fp_comparison(&all_runs, &cross_root.join("03_fp_comparison.png"))?;

    // -- Assemble Markdown report ---------------------------------------
    std::println!("[dsfb-debug-demo] Assembling Markdown report...");
    let md_path = output_root.join("report.md");
    pdf_report::assemble(&all_runs, &figs_root, &md_path)?;

    // -- Zip artefacts ---------------------------------------------------
    std::println!("[dsfb-debug-demo] Zipping artefacts...");
    let zip_path = PathBuf::from("output-dsfb-debug").join(format!("{}.zip", run_id));
    zip_dir(&output_root, &zip_path)?;

    std::println!("[dsfb-debug-demo] Done.");
    std::println!("[dsfb-debug-demo] Folder: {}", output_root.display());
    std::println!("[dsfb-debug-demo] Zip:    {}", zip_path.display());

    Ok(zip_path)
}

fn run_one_fixture(
    _label: &str,
    bytes: &[u8],
    manifest: &RealDatasetManifest,
) -> Result<FixtureRun> {
    let matrix = parse_residual_projection(bytes)?;
    if matrix.is_sentinel || matrix.num_signals == 0 {
        return Err(crate::error::DsfbError::MissingRealData);
    }
    let engine = DsfbDebugEngine::<32, 64>::paper_lock()?;
    let n = matrix.num_windows.checked_mul(matrix.num_signals).unwrap_or(usize::MAX);
    let mut eval_grid: Vec<SignalEvaluation> = std::vec![blank_eval(); n];
    let mut episodes_buf: Vec<DebugEpisode> = std::vec![blank_episode(); 256];

    let (episode_count, _metrics) = engine.run_evaluation(
        &matrix.data,
        matrix.num_signals,
        matrix.num_windows,
        &matrix.fault_labels,
        matrix.healthy_window_end,
        &mut eval_grid,
        &mut episodes_buf,
        manifest.name,
    )?;
    episodes_buf.truncate(episode_count);

    let real_data_eval = evaluate_real_dataset(&engine, manifest, bytes)?;
    let fusion_cfg = FusionConfig::ALL_DEFAULT;
    let fusion_metrics = run_fusion_evaluation(
        &engine,
        &matrix.data,
        matrix.num_signals,
        matrix.num_windows,
        matrix.healthy_window_end,
        &matrix.fault_labels,
        &fusion_cfg,
        manifest.name,
    )?;

    Ok(FixtureRun {
        manifest_name: manifest.name.to_string(),
        matrix,
        eval_grid,
        episodes: episodes_buf,
        real_data_eval,
        fusion_metrics,
    })
}

fn serialize_metrics_json(run: &FixtureRun) -> String {
    let m = &run.real_data_eval.metrics;
    let mut s = String::new();

    s.push_str("{\n");
    s.push_str(&format!("  \"manifest_name\": \"{}\",\n", run.manifest_name));
    s.push_str(&format!("  \"deterministic_replay_holds\": {},\n", run.real_data_eval.deterministic_replay_holds));
    s.push_str(&format!("  \"episode_count\": {},\n", run.real_data_eval.episode_count));

    // Fixture header (provenance)
    s.push_str("  \"fixture_header\": ");
    s.push_str(&json_string(&run.real_data_eval.fixture_header));
    s.push_str(",\n");

    // Matrix shape + channels
    s.push_str(&format!("  \"num_windows\": {},\n", run.matrix.num_windows));
    s.push_str(&format!("  \"num_signals\": {},\n", run.matrix.num_signals));
    s.push_str(&format!("  \"healthy_window_end\": {},\n", run.matrix.healthy_window_end));
    s.push_str("  \"channels\": [");
    for (i, c) in run.matrix.channels.iter().enumerate() {
        if i > 0 { s.push_str(", "); }
        s.push_str(&json_string(c));
    }
    s.push_str("],\n");

    // Fault labels per window
    s.push_str("  \"fault_labels\": [");
    for (i, &b) in run.matrix.fault_labels.iter().enumerate() {
        if i > 0 { s.push_str(","); }
        s.push_str(if b { "1" } else { "0" });
    }
    s.push_str("],\n");

    // Residual matrix flattened (row-major: window × signal)
    s.push_str("  \"residual_matrix\": [");
    for (i, &v) in run.matrix.data.iter().enumerate() {
        if i > 0 { s.push_str(","); }
        if v.is_finite() { s.push_str(&format!("{}", v)); } else { s.push_str("null"); }
    }
    s.push_str("],\n");

    // Per-(w, s) eval grid: drift_persistence, slew, grammar_state, policy_state, was_imputed
    s.push_str("  \"eval_grid\": [\n");
    for (i, ev) in run.eval_grid.iter().enumerate() {
        if i > 0 { s.push_str(",\n"); }
        s.push_str(&format!(
            "    {{\"w\":{},\"s\":{},\"r\":{},\"drift\":{},\"slew\":{},\"grammar\":\"{}\",\"reason\":\"{}\",\"policy\":\"{}\",\"imp\":{}}}",
            ev.window_index, ev.signal_index,
            if ev.residual_value.is_finite() { format!("{}", ev.residual_value) } else { "null".to_string() },
            ev.drift_persistence,
            if ev.sign_tuple.slew.is_finite() { format!("{}", ev.sign_tuple.slew) } else { "null".to_string() },
            grammar_str(ev.confirmed_grammar_state),
            reason_str(ev.reason_code),
            policy_str(ev.policy_state),
            ev.was_imputed,
        ));
    }
    s.push_str("\n  ],\n");

    // Closed episodes — full evidence packet
    s.push_str("  \"episodes\": [\n");
    for (i, ep) in run.episodes.iter().enumerate() {
        if i > 0 { s.push_str(",\n"); }
        let motif = match ep.matched_motif {
            crate::types::SemanticDisposition::Named(m) => format!("\"{:?}\"", m),
            crate::types::SemanticDisposition::Unknown => "null".to_string(),
        };
        let drift_dir = match ep.structural_signature.dominant_drift_direction {
            crate::types::DriftDirection::Positive => "Positive",
            crate::types::DriftDirection::Negative => "Negative",
            crate::types::DriftDirection::Oscillatory => "Oscillatory",
            crate::types::DriftDirection::None => "None",
        };
        let root = match ep.root_cause_signal_index {
            Some(i) => format!("{}", i),
            None => "null".to_string(),
        };
        s.push_str(&format!(
            "    {{\"id\":{},\"start_window\":{},\"end_window\":{},\"peak_grammar\":\"{}\",\"primary_reason\":\"{}\",\"matched_motif\":{},\"policy\":\"{}\",\"contributing_signals\":{},\"signature\":{{\"drift_direction\":\"{}\",\"peak_slew\":{},\"duration_windows\":{},\"signal_correlation\":{}}},\"root_cause_signal\":{}}}",
            ep.episode_id,
            ep.start_window, ep.end_window,
            grammar_str(ep.peak_grammar_state),
            reason_str(ep.primary_reason_code),
            motif,
            policy_str(ep.policy_state),
            ep.contributing_signal_count,
            drift_dir,
            ep.structural_signature.peak_slew_magnitude,
            ep.structural_signature.duration_windows,
            ep.structural_signature.signal_correlation,
            root,
        ));
    }
    s.push_str("\n  ],\n");

    // Per-detector firings (top of fusion_metrics.per_detector)
    s.push_str("  \"per_detector\": [\n");
    for (i, d) in run.fusion_metrics.per_detector.iter().enumerate() {
        if i > 0 { s.push_str(",\n"); }
        s.push_str(&format!(
            "    {{\"name\":{},\"raw_alerts\":{},\"alert_windows\":{},\"episode_count\":{},\"captured_faults\":{},\"total_faults\":{},\"clean_fp\":{},\"clean_windows\":{}}}",
            json_string(d.detector_name),
            d.raw_alert_count, d.alert_windows, d.episode_count,
            d.captured_faults, d.total_faults,
            d.clean_window_false_alerts, d.clean_windows,
        ));
    }
    s.push_str("\n  ],\n");

    // Headline metrics
    s.push_str(&format!("  \"metrics\": {{\n    \"total_windows\": {},\n    \"total_signals\": {},\n    \"raw_anomaly_count\": {},\n    \"dsfb_episode_count\": {},\n    \"rscr\": {},\n    \"episode_precision\": {},\n    \"fault_recall\": {},\n    \"investigation_load_reduction_pct\": {},\n    \"clean_window_false_episode_rate\": {}\n  }},\n",
        m.total_windows,
        m.total_signals,
        m.raw_anomaly_count,
        m.dsfb_episode_count,
        m.rscr,
        m.episode_precision,
        m.fault_recall,
        m.investigation_load_reduction_pct,
        m.clean_window_false_episode_rate,
    ));

    s.push_str(&format!("  \"fusion\": {{\n    \"detectors_used\": {},\n    \"fusion_episode_count\": {},\n    \"fusion_clean_window_fp_rate\": {},\n    \"consensus_confirmed_typed_episodes\": {},\n    \"deterministic_replay_holds\": {}\n  }}\n}}\n",
        run.fusion_metrics.detectors_used,
        run.fusion_metrics.fusion_episode_count,
        run.fusion_metrics.fusion_clean_window_fp_rate,
        run.fusion_metrics.consensus_confirmed_typed_episodes,
        run.fusion_metrics.deterministic_replay_holds,
    ));

    s
}

/// Emit canonical bank metadata as JSON for the Python figure pipeline.
fn serialize_bank_json() -> String {
    use crate::heuristics_bank::HeuristicsBank;
    let bank: HeuristicsBank<64> = HeuristicsBank::with_canonical_motifs();
    let mut s = String::from("{\n  \"motifs\": [\n");
    for (i, e) in bank.entries_iter().enumerate() {
        if i > 0 { s.push_str(",\n"); }
        let confuser = match e.confuser_motif {
            Some(m) => format!("\"{:?}\"", m),
            None => "null".to_string(),
        };
        s.push_str(&format!(
            "    {{\"motif\":\"{:?}\",\"reason_code\":\"{}\",\"provenance\":\"{:?}\",\"affinity_tiers\":{},\"primary_witness_tiers\":{},\"confuser_motif\":{},\"margin_vs_confuser_threshold\":{},\"min_correlation_count\":{},\"max_correlation_count\":{},\"min_duration_windows\":{},\"max_duration_windows\":{}}}",
            e.motif_class,
            reason_str(e.reason_code),
            e.provenance,
            e.affinity_tiers,
            e.primary_witness_tiers,
            confuser,
            e.margin_vs_confuser_threshold,
            e.min_correlation_count,
            e.max_correlation_count,
            e.min_duration_windows,
            e.max_duration_windows,
        ));
    }
    s.push_str("\n  ]\n}\n");
    s
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn grammar_str(g: GrammarState) -> &'static str {
    match g {
        GrammarState::Admissible => "Admissible",
        GrammarState::Boundary => "Boundary",
        GrammarState::Violation => "Violation",
    }
}

fn reason_str(r: ReasonCode) -> &'static str {
    match r {
        ReasonCode::Admissible => "Admissible",
        ReasonCode::SustainedOutwardDrift => "SustainedOutwardDrift",
        ReasonCode::AbruptSlewViolation => "AbruptSlewViolation",
        ReasonCode::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
        ReasonCode::DriftWithRecovery => "DriftWithRecovery",
        ReasonCode::BoundaryApproach => "BoundaryApproach",
        ReasonCode::EnvelopeViolation => "EnvelopeViolation",
        ReasonCode::SingleCrossing => "SingleCrossing",
    }
}

fn policy_str(p: PolicyState) -> &'static str {
    match p {
        PolicyState::Silent => "Silent",
        PolicyState::Watch => "Watch",
        PolicyState::Review => "Review",
        PolicyState::Escalate => "Escalate",
    }
}

fn zip_dir(src: &Path, dst: &Path) -> Result<()> {
    use std::fs::File;
    use std::io::{Read, Write};
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let file = File::create(dst).map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let prefix = src.parent().unwrap_or(Path::new(""));
    walk_and_add(&mut zip, src, prefix, opts).ok();
    zip.finish().map_err(|_| crate::error::DsfbError::ParseError { record: 0, field: 0 })?;
    Ok(())
}

fn walk_and_add(
    zip: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    strip: &Path,
    opts: zip::write::FileOptions,
) -> std::io::Result<()> {
    use std::io::Read;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(strip).unwrap_or(&path).to_string_lossy().to_string();
        if path.is_dir() {
            zip.add_directory(format!("{name}/"), opts).ok();
            walk_and_add(zip, &path, strip, opts)?;
        } else {
            zip.start_file(name, opts).ok();
            let mut f = std::fs::File::open(&path)?;
            let mut buf = std::vec::Vec::new();
            f.read_to_end(&mut buf)?;
            std::io::Write::write_all(zip, &buf)?;
        }
    }
    Ok(())
}

/// ISO-8601-ish UTC timestamp suitable for filesystem paths
/// (`YYYY-MM-DD-HHMMSS`). Hand-rolled — no `chrono` dep needed.
fn format_utc_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = unix_secs_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02}-{:02}{:02}{:02}", y, mo, d, h, mi, s)
}

/// Compute (year, month, day, hour, minute, second) UTC from
/// Unix-epoch seconds. Standard civil-date algorithm; no leap-second
/// handling (leap seconds do not occur in `SystemTime::duration_since`
/// on POSIX).
fn unix_secs_to_ymdhms(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let s = secs % 86_400;
    let h = (s / 3600) as u32;
    let mi = ((s % 3600) / 60) as u32;
    let sec = (s % 60) as u32;

    // Howard Hinnant's algorithm (civil-from-days), epoch 1970-01-01.
    let z = days + 719_468;
    let era = if z >= 0 { z / 146_097 } else { (z - 146_096) / 146_097 };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64 + era * 400) as i64;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year as u32, month as u32, day as u32, h, mi, sec)
}

fn blank_eval() -> SignalEvaluation {
    SignalEvaluation {
        window_index: 0,
        signal_index: 0,
        residual_value: 0.0,
        sign_tuple: SignTuple::ZERO,
        raw_grammar_state: GrammarState::Admissible,
        confirmed_grammar_state: GrammarState::Admissible,
        reason_code: ReasonCode::Admissible,
        motif: None,
        semantic_disposition: SemanticDisposition::Unknown,
        dsa_score: 0.0,
        policy_state: PolicyState::Silent,
        was_imputed: false,
        drift_persistence: 0.0,
    }
}

fn blank_episode() -> DebugEpisode {
    DebugEpisode {
        episode_id: 0,
        start_window: 0,
        end_window: 0,
        peak_grammar_state: GrammarState::Admissible,
        primary_reason_code: ReasonCode::Admissible,
        matched_motif: SemanticDisposition::Unknown,
        policy_state: PolicyState::Silent,
        contributing_signal_count: 0,
        structural_signature: StructuralSignature {
            dominant_drift_direction: DriftDirection::None,
            peak_slew_magnitude: 0.0,
            duration_windows: 0,
            signal_correlation: 0.0,
        },
        root_cause_signal_index: None,
    }
}
