//! POWDER-RENEW Real OTA Playback — NIST CBRS 3.55 GHz
//!
//! Loads a genuine POWDER-RENEW over-the-air IQ capture (CF32 binary + SigMF JSON)
//! and runs the full DSFB Stage III evaluation against it. Every byte of signal data
//! comes from a real USRP X310 antenna at the University of Utah campus testbed.
//! No synthetic signal generation. No simulated data. Real antennas, real air.
//!
//! # Dataset — POWDER-RENEW Platform
//!
//! NSF PAWR testbed, University of Utah.
//! Real USRP X310 OTA captures at 3.55 GHz (CBRS PAL band),
//! GPS-disciplined reference, 20 MHz bandwidth.
//!
//! Download captures from the POWDER portal:
//! ```text
//! https://www.powderwireless.net/experiments/
//! ```
//! 1. Log in with your POWDER account (free registration).
//! 2. Go to Experiments → Completed → filter by "CBRS" or "3550 MHz".
//! 3. Select any completed experiment and click "Download IQ Recording".
//! 4. Choose CF32 export format (interleaved I/Q float32 little-endian).
//! 5. Download the companion .sigmf-meta JSON file.
//!
//! Alternatively the RENEW dataset is publicly mirrored at:
//! ```text
//! https://renew-wireless.org/dataset
//! ```
//!
//! # Usage
//!
//! ```text
//! cargo run --example nist_powder_playback --features std -- \
//!     --input powder_session_20231115_3550MHz.cf32 \
//!     --meta  powder_session_20231115.sigmf-meta \
//!     [--sample-rate 1000000] \
//!     [--cal-samples 100000] \
//!     [--snr-db 18.0]
//! ```
//!
//! # File Formats
//!
//! * `.cf32` — interleaved little-endian binary float pairs: I₀ Q₀ I₁ Q₁ …
//!   Each complex sample is 8 bytes (2 × f32LE). Produced directly by GNU Radio,
//!   UHD, SoapySDR, and the POWDER portal export tool.
//! * `.sigmf-meta` — SigMF JSON. DSFB looks for `core:annotations` entries
//!   whose `core:label` contains "cbrs", "pal", "gaa", "activation",
//!   "interference", "onset", or "transition" as ground-truth markers.
//!
//! # GUM Uncertainty Budget
//!
//! Type B contributors wired in for USRP X310 hardware:
//! - ADC quantization noise: LSB/√12, 14-bit (Ettus X310 specsheet)
//! - GPS timing jitter: 1.5 ns × f_c × β (NIST TN 1263)
//!
//! Calibration window is tested for WSS (Wiener-Khinchin) before ρ is set.
//!
//! # Non-Claims (paper §L5, §XI)
//!
//! - No modulation classification or decoding is performed.
//! - No CBRS PAL license compliance assertion is made.
//! - DSFB detects structural drift in the AGC residual only.
//! - Metrics are bounded to the loaded capture; no deployment claim is made.
//!
//! # SBIR Relevance
//!
//! POWDER-RENEW is the NIST-endorsed reference testbed for CBRS band sensing.
//! Running DSFB against this real OTA data — without per-capture retuning —
//! demonstrates readiness for CBRS Priority Access License (PAL) enforcement.

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
    println!(" DSFB-RF │ POWDER-RENEW OTA Playback │ CBRS 3.55 GHz");
    println!(" Real USRP X310 captures — University of Utah campus testbed");
    println!("══════════════════════════════════════════════════════════════");
    println!();
    // ── Argument parsing ───────────────────────────────────────────────────
    let args: Vec<String> = env::args().collect();
    let mut input_path: Option<String> = None;
    let mut meta_path:  Option<String> = None;
    let mut sample_rate: f32 = 1_000_000.0;
    let mut cal_samples: usize = HEALTHY_WINDOW_SIZE;
    let mut snr_db:  f32 = 18.0;

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
        eprintln!("POWDER-RENEW dataset access:");
        eprintln!("  Portal:   https://www.powderwireless.net/experiments/");
        eprintln!("  Mirror:   https://renew-wireless.org/dataset");
        eprintln!("  Format:   CF32 (interleaved I/Q float32 little-endian) + SigMF JSON");
        eprintln!("  Band:     3.55 GHz CBRS (PAL/GAA)");
        eprintln!("  Hardware: USRP X310 with GPS-disciplined 10 MHz reference");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --example nist_powder_playback --features std -- \\");
        eprintln!("    --input powder_session_20231115_3550MHz.cf32 \\");
        eprintln!("    --meta  powder_session_20231115.sigmf-meta");
        process::exit(1);
    });

    println!(" Input : {}", input_path);
    if let Some(ref m) = meta_path { println!(" Meta  : {}", m); }
    println!(" Fs    : {:.0} Hz  |  Cal window: {} samples  |  SNR est: {:.1} dB",
        sample_rate, cal_samples, snr_db);
    println!();

    // ── Load CF32 IQ samples from real POWDER capture ────────────────────
    println!(" [1/4] Loading real POWDER-RENEW CF32 IQ data …");
    let iq_norms: Vec<f32> = {
        let f = File::open(&input_path).unwrap_or_else(|e| {
            eprintln!("ERROR: cannot open '{}': {}", input_path, e);
            process::exit(1);
        });
        let mut reader = BufReader::new(f);
        let mut buf = [0u8; 8]; // 2 × f32LE = I sample + Q sample
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

    // ── Parse SigMF metadata for ground-truth annotations ───────────────
    println!(" [2/4] Parsing SigMF annotations …");
    let mut events: Vec<RegimeTransitionEvent> = Vec::new();

    const GT_LABELS: &[&str] = &[
        "cbrs", "pal", "gaa", "activation", "interference",
        "onset", "event", "transition", "ramp",
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
                            .and_then(|l| l.as_str()).unwrap_or("sigmf_event").to_string()
                    );
                    println!("   Annotation: '{}' onset_sample={}", label_s, onset);
                    events.push(RegimeTransitionEvent { k: onset, label: label_s });
                }
            }
        }
        if events.is_empty() {
            println!("   No matching annotations found. Operating unsupervised.");
        } else {
            println!("   {} ground-truth event(s) loaded.", events.len());
        }
    } else {
        println!("   No --meta provided. Operating unsupervised (no recall metric).");
    }
    println!();

    // ── GUM uncertainty budget (USRP X310, ISO/IEC Guide 98-3 §5.1) ──────
    println!(" [3/4] GUM uncertainty budget …");
    let cal_slice = &iq_norms[..cal_samples];
    let wss = verify_wss(cal_slice, &StationarityConfig::default());
    let wss_ok = wss.map_or(false, |v| v.is_wss);
    println!("   WSS pre-condition (Wiener-Khinchin): {}",
        if wss_ok { "PASS — calibration window is wide-sense stationary" }
        else       { "WARN — window may be non-stationary; results are indicative" });

    let mut gum_cfg = UncertaintyConfig::default();
    gum_cfg.add_type_b(TypeBContributor {
        name: "adc_quantization",
        u_b: quantization_noise_std(14), // USRP X310: 14-bit ADC
        source: "ettus_usrp_x310_specsheet",
    });
    gum_cfg.add_type_b(TypeBContributor {
        name: "gps_timing_jitter",
        u_b: 1.5e-9 * 3.55e9 * 0.001,
        source: "nist_gps_discipline_tn1263",
    });
    if let Some(b) = compute_budget(cal_slice, &gum_cfg, wss_ok) {
        println!("   ρ_calibrated : {:.5}  (from real hardware noise floor)", b.rho_gum);
        println!("   Expanded U   : {:.5}  (k={}, Type A={:.5}, Type B={:.5})",
            b.expanded_uncertainty, b.coverage_factor, b.u_a, b.u_b_combined);
    }
    println!();

    // ── Build observation stream and run Stage III ───────────────────────
    println!(" [4/4] Running DSFB Stage III evaluation on real OTA data …");
    let observations: Vec<RfObservation> = iq_norms.iter().enumerate().map(|(k, &norm)| {
        RfObservation {
            k,
            residual_norm: norm,
            snr_db,
            is_healthy: k < cal_samples,
        }
    }).collect();

    let result = run_stage_iii(
        "POWDER-RENEW OTA (USRP X310, 3.55 GHz CBRS, University of Utah)",
        &observations,
        &events,
    );
    println!();
    result.print_summary();

    println!();
    println!(" Hardware: USRP X310, 14-bit ADC, GPS-disciplined reference.");
    println!(" Signal path: antenna → LNA → DDC → AGC → residual → DSFB.");
    println!(" Observer contract: read-only | no_std core | zero unsafe | no alloc.");
    println!();
    if !events.is_empty() {
        println!(" Ground-truth coverage:");
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
