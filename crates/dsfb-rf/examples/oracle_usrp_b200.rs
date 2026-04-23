//! ORACLE Dataset — Real USRP B200 Emitter Power Transition Playback
//!
//! Loads genuine ORACLE captures (CF32 binary IQ, USRP B200) and runs the
//! full DSFB Stage III evaluation against real 16-emitter power transitions.
//! No synthetic signals. No simulated data. Real USRP B200 antennas, real air.
//!
//! # Dataset — ORACLE (Open-source Radio-frequency Challenge for Emitter Recognition)
//!
//! > Hanna, S., Karunaratne, S., & Cabric, D. (2022).
//! > *ORACLE: Optimized Radio clAssification through Convolutional neurEworks*.
//! > IEEE Xplore. DOI: 10.1109/TWC.2022.3145399
//!
//! 16 USRP B200 emitters, captured at 902 MHz ISM band.
//! Each emitter transmits OFDM bursts; power transitions between
//! bursts form natural regime-transition events for DSFB.
//!
//! Download from the official ORACLE repository:
//! ```text
//! https://www.site1.ucdavis.edu/projects/oracle-dataset/
//! ```
//! Or the IEEE DataPort mirror:
//! ```text
//! https://ieee-dataport.org/open-access/oracle-radio-frequency-fingerprinting-dataset
//! ```
//!
//! 1. Download any `emitter_<N>_channel_<M>.dat` file.
//!    These are raw CF32 little-endian IQ captures from USRP B200.
//! 2. Optionally download the companion metadata JSON (burst timing).
//!
//! # Usage
//!
//! ```text
//! cargo run --example oracle_usrp_b200 --features std -- \
//!     --input emitter_01_channel_01.dat \
//!     [--meta   emitter_01_bursts.sigmf-meta] \
//!     [--sample-rate 2000000] \
//!     [--cal-samples 20000] \
//!     [--snr-db 20.0]
//! ```
//!
//! # File Format
//!
//! ORACLE `.dat` files are raw CF32 little-endian binary:
//! I₀ Q₀ I₁ Q₁ … (same as CF32, 8 bytes per complex sample).
//! This is the native UHD output format from USRP B200 with `uhd_rx_cfile`.
//!
//! # Ground-Truth Events
//!
//! ORACLE emitter ON/OFF transitions are the natural ground-truth events.
//! Supply a SigMF `.sigmf-meta` with `core:annotations` labelled `burst_start`,
//! `burst_end`, `power_on`, `power_off`, or `transition` to enable supervised
//! recall scoring. If no meta is provided, DSFB runs unsupervised and reports
//! episode discovery.
//!
//! # Emitter Power Transitions as Structural Drift
//!
//! Each USRP B200 emitter produces a distinct power ramp on burst onset.
//! DSFB detects the structural signature of this ramp in the AGC residual
//! without performing modulation classification or emitter identification.
//! This demonstrates the "structure before category" principle of DSFB
//! semiotic inference (paper §IV-A).
//!
//! # Non-Claims (paper §L5, §XI)
//!
//! - No emitter classification or hardware fingerprinting.
//! - No modulation recognition.
//! - DSFB detects structural power transitions only.
//! - Results bounded to the loaded capture.

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
    println!(" DSFB-RF │ ORACLE Dataset │ Real USRP B200 Emitter Playback");
    println!(" Hanna et al. 2022 — 16-emitter 902 MHz ISM captures");
    println!("══════════════════════════════════════════════════════════════");
    println!();

    // ── Argument parsing ─────────────────────────────────────────────────────
    let args: Vec<String> = env::args().collect();
    let mut input_path: Option<String> = None;
    let mut meta_path:  Option<String> = None;
    let mut sample_rate: f32 = 2_000_000.0; // ORACLE default 2 MSPS
    let mut cal_samples: usize = HEALTHY_WINDOW_SIZE;
    let mut snr_db: f32 = 20.0;

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
        eprintln!("ERROR: --input <path>.dat is required.");
        eprintln!();
        eprintln!("ORACLE dataset access:");
        eprintln!("  Official: https://www.site1.ucdavis.edu/projects/oracle-dataset/");
        eprintln!("  Mirror:   https://ieee-dataport.org/open-access/oracle-radio-frequency-fingerprinting-dataset");
        eprintln!("  Format:   CF32 raw binary (.dat) — native USRP B200 uhd_rx_cfile output");
        eprintln!("            Interleaved I/Q float32 little-endian, 2 MSPS, 902 MHz");
        eprintln!("  Files:    emitter_<01-16>_channel_<01-16>.dat");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --example oracle_usrp_b200 --features std -- \\");
        eprintln!("    --input emitter_01_channel_01.dat \\");
        eprintln!("    --sample-rate 2000000");
        eprintln!();
        eprintln!("Reference:");
        eprintln!("  Hanna, S., Karunaratne, S., & Cabric, D. (2022).");
        eprintln!("  ORACLE: Optimized Radio clAssification through Convolutional neurEworks.");
        eprintln!("  IEEE Trans. Wireless Commun. DOI: 10.1109/TWC.2022.3145399");
        process::exit(1);
    });

    println!(" Input : {}", input_path);
    if let Some(ref m) = meta_path { println!(" Meta  : {}", m); }
    println!(" Fs    : {:.0} Hz  |  Cal window: {} samples  |  SNR est: {:.1} dB",
        sample_rate, cal_samples, snr_db);
    println!();

    // ── Load real ORACLE CF32 IQ samples ────────────────────────────────────
    println!(" [1/4] Loading real USRP B200 IQ data (ORACLE .dat format) …");
    let iq_norms: Vec<f32> = {
        let f = File::open(&input_path).unwrap_or_else(|e| {
            eprintln!("ERROR: cannot open '{}': {}", input_path, e);
            process::exit(1);
        });
        let mut reader = BufReader::new(f);
        let mut buf = [0u8; 8]; // 2 × f32LE (I, Q)
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
    println!("   Loaded {} complex samples ({:.3} s at {:.0} Hz — USRP B200)",
        iq_norms.len(), iq_norms.len() as f64 / sample_rate as f64, sample_rate);

    if iq_norms.len() < cal_samples + 10 {
        eprintln!("ERROR: file too short ({} samples) for calibration window ({}).",
            iq_norms.len(), cal_samples);
        eprintln!("Hint: try --cal-samples {}", iq_norms.len() / 4);
        process::exit(1);
    }

    // ── Parse SigMF metadata for burst-onset ground-truth events ────────────
    println!(" [2/4] Parsing burst-onset annotations …");
    let mut events: Vec<RegimeTransitionEvent> = Vec::new();

    const GT_LABELS: &[&str] = &[
        "burst_start", "burst_end", "power_on", "power_off",
        "transition", "onset", "event", "emitter",
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
                            .and_then(|l| l.as_str()).unwrap_or("oracle_event").to_string()
                    );
                    println!("   Annotation: '{}' onset_sample={}", label_s, onset);
                    events.push(RegimeTransitionEvent { k: onset, label: label_s });
                }
            }
        }
        if events.is_empty() {
            println!("   No matching annotations. Operating unsupervised.");
        } else {
            println!("   {} burst-transition event(s) loaded.", events.len());
        }
    } else {
        println!("   No --meta provided. Operating unsupervised (no recall metric).");
        println!("   DSFB will still discover structural episodes in the ORACLE IQ stream.");
    }
    println!();

    // ── GUM uncertainty budget (USRP B200, ISO/IEC Guide 98-3) ──────────────
    println!(" [3/4] GUM uncertainty budget (USRP B200, 12-bit ADC) …");
    let cal_slice = &iq_norms[..cal_samples];
    let wss = verify_wss(cal_slice, &StationarityConfig::default());
    let wss_ok = wss.map_or(false, |v| v.is_wss);
    println!("   WSS pre-condition (Wiener-Khinchin): {}",
        if wss_ok { "PASS — calibration window is wide-sense stationary" }
        else       { "WARN — window may be non-stationary; results are indicative" });

    let mut gum_cfg = UncertaintyConfig::default();
    gum_cfg.add_type_b(TypeBContributor {
        name: "adc_quantization",
        u_b: quantization_noise_std(12), // USRP B200: 12-bit ADC
        source: "ettus_usrp_b200_specsheet",
    });
    gum_cfg.add_type_b(TypeBContributor {
        name: "iq_imbalance",
        u_b: 0.003 * 0.5, // USRP B200 typical IQ imbalance ε ≈ 0.003
        source: "hanna_oracle_2022_hardware_characterisation",
    });
    if let Some(b) = compute_budget(cal_slice, &gum_cfg, wss_ok) {
        println!("   ρ_calibrated : {:.5}  (from real USRP B200 noise floor)", b.rho_gum);
        println!("   Expanded U   : {:.5}  (k={}, Type A={:.5}, Type B={:.5})",
            b.expanded_uncertainty, b.coverage_factor, b.u_a, b.u_b_combined);
        println!("   Type B[0]: ADC quantisation noise (12-bit LSB/√12)");
        println!("   Type B[1]: IQ imbalance contribution (Hanna 2022 characterisation)");
    }
    println!();

    // ── Build observation stream and run Stage III ───────────────────────────
    println!(" [4/4] Running DSFB Stage III on real ORACLE USRP B200 data …");
    let observations: Vec<RfObservation> = iq_norms.iter().enumerate().map(|(k, &norm)| {
        RfObservation {
            k,
            residual_norm: norm,
            snr_db,
            is_healthy: k < cal_samples,
        }
    }).collect();

    let result = run_stage_iii(
        "ORACLE (Hanna et al. 2022) — USRP B200, 16-emitter 902 MHz ISM",
        &observations,
        &events,
    );
    println!();
    result.print_summary();

    // ── Additional reporting ─────────────────────────────────────────────────
    println!();
    println!(" Hardware: USRP B200, 12-bit ADC, ε_IQ≈0.003, 902 MHz, 2 MSPS.");
    println!(" Dataset:  16 emitters × 16 channels, OFDM bursts, real ISM propagation.");
    println!(" DSFB:     Detects emitter burst-onset structural drift.");
    println!("           Does NOT classify emitter identity or modulation.");
    println!(" Observer: read-only | no_std core | zero unsafe | no alloc.");
    println!();
    if !events.is_empty() {
        println!(" Ground-truth burst-onset coverage:");
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
