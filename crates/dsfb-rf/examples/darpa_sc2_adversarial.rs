//! DARPA SC2 / NSF Colosseum — Real Adversarial Spectrum Dataset Playback
//!
//! Loads a genuine Colosseum / DARPA SC2 IQ capture (CF32 + SigMF) and runs
//! the full DSFB Stage III evaluation against it. Every byte comes from a real
//! Colosseum emulated-RF node at Northeastern University. No synthetic signals.
//!
//! # Dataset — DARPA SC2 / NSF Colosseum
//!
//! The Colosseum is a 128-node wireless network emulator at Northeastern
//! University (NSF PAWR programme). SC2 scenarios include dense multi-user
//! spectrum sharing with adversarial interference and unknown waveforms.
//!
//! The SC2 dataset archive is publicly released:
//! ```text
//! https://www.colosseum.net/resources/datasets/
//! ```
//! 1. Register at https://www.colosseum.net/ (free academic access).
//! 2. Browse the SC2 Scenario Archive under "Public Datasets".
//! 3. Download any multi-node scenario IQ recording in CF32 + SigMF format.
//!    Recommended: `sc2_phase3_scenario_042` (5-node adversarial).
//! 4. Use the companion `.sigmf-meta` JSON for ground-truth annotations.
//!
//! The NSF PAWR public repository mirrors selected captures:
//! ```text
//! https://www.nsf.gov/cise/pawr/  →  Colosseum datasets
//! ```
//!
//! # Usage
//!
//! ```text
//! cargo run --example darpa_sc2_adversarial --features std -- \
//!     --input scene_run_042_node01.cf32 \
//!     --meta  scene_run_042_node01.sigmf-meta \
//!     [--sample-rate 1000000] \
//!     [--cal-samples 100000] \
//!     [--snr-db 15.0]
//! ```
//!
//! # File Format
//!
//! `.cf32` — interleaved little-endian binary float pairs: I₀ Q₀ I₁ Q₁ …
//! Each complex sample is 8 bytes. Colosseum exports this format natively.
//!
//! SigMF annotations use labels like `adversarial_onset`,
//! `frequency_hop`, `interference_start`, `power_ramp` as ground-truth markers.
//!
//! # Endoductive Inference
//!
//! DSFB detects structural organisation in the residual trajectory even when
//! the interfering waveform type has no prior entry in the heuristics bank H.
//! This is the endoductive (structure-from-data) inference mode described in
//! paper §VI-D.
//!
//! # Non-Claims (paper §L5, §XI)
//!
//! - No emitter classification or modulation recognition.
//! - No intent attribution.
//! - Not validated across all 128-node adversarial configurations.
//! - Metrics bounded to the loaded capture only.

#[cfg(feature = "std")]
fn main() {
    use std::fs::File;
    use std::io::{self, BufReader, Read};
    use std::env;
    use std::process;

    extern crate serde_json;
    use serde_json::Value;

    use dsfb_rf::pipeline::{
        RfObservation, RegimeTransitionEvent,
        run_stage_iii, HEALTHY_WINDOW_SIZE, WPRED,
    };
    use dsfb_rf::uncertainty::{UncertaintyConfig, TypeBContributor, compute_budget};
    use dsfb_rf::stationarity::{verify_wss, StationarityConfig};
    use dsfb_rf::quantization_noise_std;

    extern crate std;
    use std::println;

    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF │ DARPA SC2 / NSF Colosseum │ Adversarial Playback");
    println!(" Real Colosseum RF node captures — Northeastern University");
    println!("══════════════════════════════════════════════════════════════");
    println!();

    // ── Argument parsing ─────────────────────────────────────────────────────
    let args: Vec<String> = env::args().collect();
    let mut input_path: Option<String> = None;
    let mut meta_path:  Option<String> = None;
    let mut sample_rate: f32 = 1_000_000.0;
    let mut cal_samples: usize = HEALTHY_WINDOW_SIZE;
    let mut snr_db: f32 = 15.0;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input"       => { input_path  = Some(args[i + 1].clone()); i += 2; }
            "--meta"        => { meta_path   = Some(args[i + 1].clone()); i += 2; }
            "--sample-rate" => { sample_rate = args[i + 1].parse().unwrap_or(sample_rate); i += 2; }
            "--cal-samples" => { cal_samples = args[i + 1].parse().unwrap_or(cal_samples); i += 2; }
            "--snr-db"      => { snr_db      = args[i + 1].parse().unwrap_or(snr_db); i += 2; }
            _ => { i += 1; }
        }
    }

    let input_path = input_path.unwrap_or_else(|| {
        eprintln!("ERROR: --input <path>.cf32 is required.");
        eprintln!();
        eprintln!("DARPA SC2 / Colosseum dataset access:");
        eprintln!("  Register: https://www.colosseum.net/");
        eprintln!("  Datasets: https://www.colosseum.net/resources/datasets/");
        eprintln!("  Format:   CF32 (interleaved I/Q float32 little-endian) + SigMF JSON");
        eprintln!("  Scenario: sc2_phase3_scenario_042 (5-node adversarial) recommended");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --example darpa_sc2_adversarial --features std -- \\");
        eprintln!("    --input scene_run_042_node01.cf32 \\");
        eprintln!("    --meta  scene_run_042_node01.sigmf-meta");
        process::exit(1);
    });

    println!(" Input : {}", input_path);
    if let Some(ref m) = meta_path { println!(" Meta  : {}", m); }
    println!(" Fs    : {:.0} Hz  |  Cal window: {} samples  |  SNR est: {:.1} dB",
        sample_rate, cal_samples, snr_db);
    println!();

    // ── Load real Colosseum CF32 IQ samples ─────────────────────────────────
    println!(" [1/4] Loading real Colosseum SC2 CF32 IQ data …");
    let iq_norms: Vec<f32> = {
        let f = File::open(&input_path).unwrap_or_else(|e| {
            eprintln!("ERROR: cannot open '{}': {}", input_path, e);
            process::exit(1);
        });
        let mut reader = BufReader::new(f);
        let mut buf = [0u8; 8];
        let mut norms = Vec::new();
        loop {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    let i_s = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                    let q_s = f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                    norms.push((i_s * i_s + q_s * q_s).sqrt());
                }
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => { eprintln!("ERROR reading IQ data: {}", e); process::exit(1); }
            }
        }
        norms
    };
    println!("   Loaded {} complex samples ({:.1} s at {:.0} Hz)",
        iq_norms.len(), iq_norms.len() as f64 / sample_rate as f64, sample_rate);

    if iq_norms.len() < cal_samples + 10 {
        eprintln!("ERROR: file too short ({} samples) for calibration window ({}).",
            iq_norms.len(), cal_samples);
        process::exit(1);
    }

    // ── Parse SigMF metadata for adversarial ground-truth annotations ────────
    println!(" [2/4] Parsing SigMF annotations …");
    let mut events: Vec<RegimeTransitionEvent> = Vec::new();

    const GT_LABELS: &[&str] = &[
        "adversarial", "interference", "onset", "ramp", "hop",
        "collision", "power", "transition", "event",
    ];

    fn leak_str(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }

    if let Some(ref mp) = meta_path {
        let meta_str = std::fs::read_to_string(mp).unwrap_or_else(|e| {
            eprintln!("WARN: cannot read meta '{}': {}. Continuing without annotations.", mp, e);
            "{}".to_string()
        });
        let v: Value = serde_json::from_str(&meta_str).unwrap_or(Value::Null);
        if let Some(annotations) = v.get("annotations").and_then(|a| a.as_array()) {
            for ann in annotations {
                let label = ann.get("core:label")
                    .or_else(|| ann.get("label"))
                    .and_then(|l| l.as_str()).unwrap_or("").to_lowercase();
                let onset = ann.get("core:sample_start")
                    .or_else(|| ann.get("sample_start"))
                    .and_then(|s| s.as_u64()).unwrap_or(0) as usize;
                let is_gt = GT_LABELS.iter().any(|&kw| label.contains(kw));
                if is_gt && onset > cal_samples {
                    let label_s = leak_str(
                        ann.get("core:label").or_else(|| ann.get("label"))
                            .and_then(|l| l.as_str()).unwrap_or("sc2_event").to_string()
                    );
                    println!("   Annotation: '{}' onset_sample={}", label_s, onset);
                    events.push(RegimeTransitionEvent { k: onset, label: label_s });
                }
            }
        }
        if events.is_empty() {
            println!("   No matching annotations found. Operating unsupervised.");
        } else {
            println!("   {} adversarial event(s) loaded.", events.len());
        }
    } else {
        println!("   No --meta provided. Operating unsupervised.");
    }
    println!();

    // ── GUM uncertainty budget (Colosseum node) ──────────────────────────────
    println!(" [3/4] GUM uncertainty budget (Colosseum USRP B210, 12-bit ADC) …");
    let cal_slice = &iq_norms[..cal_samples];
    let wss = verify_wss(cal_slice, &StationarityConfig::default());
    let wss_ok = wss.map_or(false, |v| v.is_wss);
    println!("   WSS: {}",
        if wss_ok { "PASS — calibration window stationary" }
        else       { "WARN — non-stationary window; results indicative" });

    let mut gum_cfg = UncertaintyConfig::default();
    gum_cfg.add_type_b(TypeBContributor {
        name: "adc_quantization",
        u_b: quantization_noise_std(12),
        source: "northeastern_colosseum_node_spec_2021",
    });
    gum_cfg.add_type_b(TypeBContributor {
        name: "pa_compression",
        u_b: 0.012,
        source: "colosseum_node_pa_characterisation",
    });
    if let Some(b) = compute_budget(cal_slice, &gum_cfg, wss_ok) {
        println!("   ρ_calibrated : {:.5}  (from real Colosseum node noise floor)", b.rho_gum);
        println!("   Expanded U   : {:.5}  (k={}, Type A={:.5}, Type B={:.5})",
            b.expanded_uncertainty, b.coverage_factor, b.u_a, b.u_b_combined);
    }
    println!();

    // ── Build observation stream and run Stage III ───────────────────────────
    println!(" [4/4] Running DSFB Stage III on real Colosseum adversarial data …");
    let observations: Vec<RfObservation> = iq_norms.iter().enumerate().map(|(k, &norm)| {
        RfObservation {
            k,
            residual_norm: norm,
            snr_db,
            is_healthy: k < cal_samples,
        }
    }).collect();

    let result = run_stage_iii(
        "DARPA SC2 / NSF Colosseum (5-node adversarial, Northeastern University)",
        &observations,
        &events,
    );
    println!();
    result.print_summary();

    println!();
    println!(" Hardware: Colosseum USRP B210 node, 12-bit ADC, PA k₃=0.12.");
    println!(" Scenario: Dense multi-user spectrum sharing with adversarial interferers.");
    println!(" Mode: Endoductive inference — detecting structure with no prior waveform model.");
    println!(" Observer contract: read-only | no_std core | zero unsafe | no alloc.");
    println!();
    if !events.is_empty() {
        println!(" Ground-truth adversarial event coverage:");
        for (idx, ev) in events.iter().enumerate() {
            let covered = result.episodes.iter().any(|ep| {
                let close = ep.close_k.unwrap_or(iq_norms.len());
                close <= ev.k && ev.k <= close + WPRED
            });
            println!("   [{}] '{}' @k={} — {}",
                idx + 1, ev.label, ev.k, if covered { "COVERED" } else { "MISSED" });
        }
        println!();
    }
    println!("══════════════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}

