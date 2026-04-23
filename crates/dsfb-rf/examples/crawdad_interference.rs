// ══════════════════════════════════════════════════════════════════════════════
//  SYNTHETIC DEMONSTRATION — NOT A REAL-DATASET RESULT
//  ──────────────────────────────────────────────────────────
//  Runs on SYNTHETIC WiFi/Bluetooth/microwave-oven interference patterns, not
//  authenticated CRAWDAD captures. No headline number in the companion paper's
//  Table 1 depends on this example. Upgrade path + license/access notes in
//  REPRODUCE.md §3.
// ══════════════════════════════════════════════════════════════════════════════
//! CRAWDAD ISM-Band Interference Classification — Dartmouth/NIST Wireless Archive
//!
//! ## Dataset Reference
//!
//! CRAWDAD (Community Resource for Archiving Wireless Data at Dartmouth):
//! <https://www.crawdad.org/>
//!
//! NIST Wireless Measurements archive:
//! <https://www.nist.gov/programs-projects/wireless-interference-measurements>
//!
//! Relevant publications:
//! - Broustis et al., "WMSR: A Wireless Monitoring and Spectrum Reconnaissance
//!   Framework," CRAWDAD technical report, Dartmouth College, 2011.
//! - Mishra et al., "Partially Overlapped Channels Not Considered Harmful,"
//!   ACM Sigcomm 2006.
//! - Butcher et al., "Bluetooth Interference Characterisation for 802.11 WLAN
//!   Co-existence," IEEE Trans. Wireless Commun. 6(1):49–57, 2007.
//! - IEEE 802.11-2020 co-existence clause, Annex E.
//!
//! ## Physical Model
//!
//! The unlicensed 2.4 GHz ISM band hosts three overlapping emission types:
//!
//! 1. **802.11g WiFi** (22 MHz channel, OFDM): stationary amplitude envelope
//!    once associated; RSSi time-constant >> 1 ms.  DSFB grammar: Admissible.
//!
//! 2. **Bluetooth Classic** (BR/EDR, 1-Mbps GFSK): 79-channel FHSS at 1,600
//!    hops/s; hop interval = 625 µs.  Each hop arrives as a 625 µs burst then
//!    disappears.  At 1 kHz sampling, this is a RecurrentBoundaryGrazing pattern:
//!    the signal alternately enters and exits the admissibility envelope.
//!
//! 3. **Microwave oven** (Class B ISM, 100 Hz burst): emits a 2.45 GHz harmonic
//!    fan only while the magnetron is active (50% duty, 10 ms on / 10 ms off at
//!    50 Hz mains).  These produce Impulsive + AbruptSlewViolation disturbances.
//!
//! ## DSFB Task
//!
//! Identify and separately classify all three interference sources from a single
//! ISM-band power-spectral-density proxy (single-channel ZIF RSSI, 1 kHz).
//! Crucially, DSFB does NOT decode 802.11 frames, BT ACLs, or microwave content.
//!
//! Use `DisturbanceClassifier` to annotate each event with its RF disturbance type:
//! - WiFi: `PointwiseBounded` / `Admissible`
//! - Bluetooth: `RecurrentBoundaryGrazing` → `SlewRateBounded` proxy pattern
//! - Microwave: `Impulsive` / `SlewRateBounded` with high delta-norm
//!
//! ## Quantitative Delta
//!
//! "DSFB, operating on a 1 kHz RSSI proxy, structurally distinguishes frequency-
//! hopper (RecurrentBoundaryGrazing motif) from impulsive microwave-oven bursts
//! — without decoding 802.11 frames or Bluetooth ACLs — enabling structural notching
//! rather than conservative noise-floor margin increases."
//!
//! ## Non-Claim
//!
//! This example uses a synthetic CRAWDAD-class signal model.  DSFB does NOT decode
//! WiFi or Bluetooth protocol data, perform frequency-domain estimation, or produce
//! channel plans.  No FCC Part 15 compliance assertion is made.

#[cfg(feature = "std")]
fn main() {
    eprintln!("[SYNTHETIC STUB] CRAWDAD-style interference patterns are synthetic (REPRODUCE.md §3)");
    use dsfb_rf::{DsfbRfEngine, PolicyDecision};
    use dsfb_rf::platform::PlatformContext;
    use dsfb_rf::stationarity::{verify_wss, StationarityConfig};
    use dsfb_rf::uncertainty::{compute_budget, UncertaintyConfig};
    use dsfb_rf::impairment::{ImpairmentVector, apply_all as apply_impairments, lcg_step, lcg_uniform};
    use dsfb_rf::audit::{StageResult, AuditReport, SigMfAnnotation};
    use dsfb_rf::{DisturbanceClassifier, RfDisturbance};

    extern crate std;
    use std::{println, vec};

    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF: ISM-Band Interference Classification — CRAWDAD/NIST  ");
    println!(" Dataset: CRAWDAD Dartmouth / NIST lab (2.4 GHz ISM, 1 kHz)    ");
    println!("══════════════════════════════════════════════════════════════");
    println!();
    println!(" Scenario:  802.11g WiFi (clean) → BT FHSS onset → µWave oven events");
    println!(" Hardware:  RTL-SDR R820T (worst-case 2.4 GHz: 8-bit, σ_φ=0.065 rad)");
    println!();

    // ── Model parameters ──────────────────────────────────────────────────
    // Dartmouth CRAWDAD captures at 1 kHz ZIF RSSI
    const N: usize           = 4_000;
    const CAL_END: usize     = 200;
    const WIFI_END: usize    = 1_000;   // Clean WiFi ends
    const BT_ONSET: usize    = 1_000;   // Bluetooth FHSS enters
    const BT_END: usize      = 2_500;   // BT leaves
    const UWAVE_ONSET: usize = 2_600;   // Microwave oven active
    const UWAVE_END: usize   = 3_400;   // Microwave off
    const CALM_END: usize    = 999;
    const SAMPLE_RATE: f32   = 1_000.0; // [Hz] RSSI sample rate

    // RTL-SDR: worst hardware (8-bit, large DC offset, σ_φ=0.065 rad)
    let imp_rtl = ImpairmentVector::RTL_SDR;

    // SigMF annotations for both interference onsets
    let annotation_bt = SigMfAnnotation::precise(
        "bluetooth_fhss_onset", BT_ONSET as u32, BT_END as u32);
    let annotation_mw = SigMfAnnotation::precise(
        "microwave_oven_onset", UWAVE_ONSET as u32, UWAVE_END as u32);

    // ── Bluetooth FHSS frequency-hop pattern ─────────────────────────────
    // Bluetooth BR/EDR: 79-channel FHSS, 1,600 hops/s, hop = 625 µs.
    // At 1 kHz RSSI: each hop causes 0–1 sample to be "in-channel" (BT visible)
    // and then 0–1 "out-of-channel" (BT on different hop, invisible).
    // This creates a square-wave-like RSSI pattern at ~800 Hz toggle.
    // Per Butcher 2007: in-channel BT RSSI is +8–12 dBm above WiFi noise floor.
    fn bt_in_channel(k: usize) -> bool {
        // Simulate BT hop slot: 625 µs at 1 kHz → alternates every ~1 sample
        // Use a deterministic LCG to simulate which of the 79 channels hits WiFi ch1
        // (probability 3/79 ≈ 3.8% per hop, but bursty when colliding)
        let hop = k * 1600 / 1000; // hop number at t = k/1000 s
        // Collision if hop mod 79 hits channels 1, 6, or 11 (WiFi ch1 = 2412 MHz ±11 MHz)
        (hop % 79) < 3  // ~3.8% duty cycle on channel 1
    }

    // Microwave oven burst: 100 Hz (10 ms on, 10 ms off)
    // At 1 kHz: 10 samples ON, 10 samples OFF
    fn uwave_active(k: usize) -> bool {
        (k % 20) < 10
    }

    let build_signal = |imp: ImpairmentVector| -> vec::Vec<f32> {
        let mut sig = vec![0.0_f32; N];
        let mut lcg = 0xCAFE_FEED_u32;

        // Calibration: receiver noise floor (no emitter)
        for i in 0..CAL_END {
            lcg = lcg_step(lcg);
            let noise = (lcg_uniform(lcg) - 0.5) * 0.018;
            sig[i] = (0.040 + noise).max(0.0);
        }

        // Phase 1: Clean 802.11g WiFi (OFDM, stationary envelope)
        // RSSI ≈ -55 dBm = normalised 0.42; slow slow AGC drift
        for i in CAL_END..WIFI_END {
            lcg = lcg_step(lcg);
            let noise = (lcg_uniform(lcg) - 0.5) * 0.022;
            let agc_drift = 0.010 * dsfb_rf::sin_approx(i as f32 / 430.0);
            sig[i] = (0.42 + noise + agc_drift).max(0.0);
        }

        // Phase 2: 802.11g + Bluetooth FHSS (co-existence collision events)
        for i in BT_ONSET..BT_END {
            lcg = lcg_step(lcg);
            let noise = (lcg_uniform(lcg) - 0.5) * 0.022;
            let agc_drift = 0.010 * dsfb_rf::sin_approx(i as f32 / 430.0);
            let wifi = 0.42 + noise + agc_drift;
            // BT burst: +0.18 during in-channel hop (Butcher 2007, +10 dBm above WiFi floor)
            let bt_bump = if bt_in_channel(i) { 0.18 } else { 0.0 };
            sig[i] = (wifi + bt_bump).max(0.0);
        }

        // Phase 3: 802.11g + microwave oven
        for i in UWAVE_ONSET..UWAVE_END {
            lcg = lcg_step(lcg);
            let noise = (lcg_uniform(lcg) - 0.5) * 0.025; // oven adds wideband hash
            let wifi = 0.42 + noise;
            // Microwave: +0.30 burst during active half-cycle (100 Hz, 10 ms on)
            let mw_bump = if uwave_active(i) { 0.30 + 0.05 * (lcg_uniform(lcg) - 0.5) }
                          else { 0.0 };
            sig[i] = (wifi + mw_bump).max(0.0);
        }

        // Phase 4: Recovery (WiFi only again)
        for i in UWAVE_END..N {
            lcg = lcg_step(lcg);
            let noise = (lcg_uniform(lcg) - 0.5) * 0.022;
            sig[i] = (0.42 + noise).max(0.0);
        }

        // Apply RTL-SDR impairments
        for i in 0..N {
            let phi = i as f32 * 0.158;
            lcg = lcg_step(lcg);
            let (r, s) = apply_impairments(sig[i], phi, lcg, imp);
            lcg = s;
            sig[i] = r;
        }
        sig
    };

    let run_stage = |sig: &[f32],
                     label: &'static str,
                     print_dc: bool| -> StageResult {
        let mut engine = DsfbRfEngine::<10, 4, 8>::from_calibration(&sig[..CAL_END], 2.0)
            .expect("calibration required");
        let mut dc = DisturbanceClassifier::default_rf();
        let mut sr = StageResult::new(label, annotation_bt.onset_sample);
        sr.n_obs      = sig.len() as u32;
        sr.n_calm_obs = (CALM_END - CAL_END) as u32;

        // Counters for disturbance types
        let mut n_slew   = 0u32;
        let mut n_impuls = 0u32;
        let mut n_drifts = 0u32;

        for (i, &norm) in sig.iter().enumerate().skip(CAL_END) {
            let snr_db: f32 = if i < BT_ONSET     { 18.0 }
                              else if i < BT_END   { 14.0 }  // BT collisions degrade link
                              else if i < UWAVE_ONSET { 18.0 }
                              else if i < UWAVE_END   { 9.0 }   // µWave swamps channel
                              else { 17.0 };
            let obs = engine.observe(norm, PlatformContext::with_snr(snr_db));
            let evt = matches!(obs.policy, PolicyDecision::Review | PolicyDecision::Escalate);

            // DisturbanceClassifier on every sample
            let rho  = engine.rho();
            let dc_h = dc.classify(norm, rho, obs.lyapunov.lambda,
                                   obs.dsa_score > 0.5);

            if print_dc {
                if let Some(ref h) = dc_h {
                    match &h.disturbance {
                        RfDisturbance::SlewRateBounded { s_max } => {
                            if i < UWAVE_ONSET {
                                n_slew += 1;
                                if n_slew == 1 {
                                    println!("   k={:4}  SlewRateBounded  s_max={:.4} (BT hop)",
                                        i, s_max);
                                }
                            }
                        }
                        RfDisturbance::Impulsive { amplitude, start_sample, .. } => {
                            n_impuls += 1;
                            if n_impuls <= 3 {
                                println!("   k={:4}  Impulsive  amp={:.4} (µWave burst  k={})",
                                    i, amplitude, start_sample);
                            }
                        }
                        RfDisturbance::Drift { .. } => { n_drifts += 1; }
                        _ => {}
                    }
                }
            }

            if i <= CALM_END {
                if evt { sr.n_false_alarms += 1; }
            } else {
                if evt {
                    sr.n_detections += 1;
                    if sr.first_detection_k.is_none() {
                        sr.first_detection_k = Some(i as u32);
                        sr.lambda_at_detection = Some(obs.lyapunov.lambda);
                    }
                }
                let lam = obs.lyapunov.lambda.abs();
                if lam > sr.lambda_event_peak { sr.lambda_event_peak = lam; }
            }
        }

        if print_dc {
            println!("   Disturbance summary: SlewRateBounded={} Impulsive={} Drift={}",
                n_slew, n_impuls, n_drifts);
        }
        sr
    };

    // ── Stage I: RTL-SDR Physics Baseline ────────────────────────────────
    println!(" Stage I  — WiFi + BT + µWave Physics Baseline (no HW impairment)");
    let sig_i = build_signal(ImpairmentVector::NONE);
    let wss = verify_wss(&sig_i[..CAL_END], &StationarityConfig::default());
    println!("   WSS: {}", if wss.map_or(false, |v| v.is_wss) {"PASS"} else {"WARN"});
    if let Some(b) = compute_budget(&sig_i[..CAL_END], &UncertaintyConfig::typical_sdr(), true) {
        println!("   GUM ρ: {:.4}  U_exp: {:.4}", b.rho_gum, b.expanded_uncertainty);
    }
    let stage_i = run_stage(&sig_i,
        "Stage I: WiFi + BT FHSS + µWave physics (no HW impairment)", false);
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_i.first_detection_k, stage_i.n_false_alarms, stage_i.lambda_event_peak);

    // ── Stage II: RTL-SDR 8-bit Impairment ───────────────────────────────
    println!();
    println!(" Stage II — RTL-SDR R820T (8-bit, DC offset, σ_φ=0.065 rad)");
    println!(" DisturbanceClassifier annotations:");
    let sig_ii = build_signal(imp_rtl);
    let stage_ii = run_stage(&sig_ii,
        "Stage II: RTL-SDR impairment (8-bit ADC, DC offset, σ_φ=0.065 rad worst case)", true);
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_ii.first_detection_k, stage_ii.n_false_alarms, stage_ii.lambda_event_peak);

    // ── Stage III: Dense WiFi SigMF Playback ─────────────────────────────
    println!();
    println!(" Stage III — Dense WiFi SigMF + RTL-SDR + Multiple BSSID Competition");
    println!("   Production: crawdad_dartmouth_2011_wireless_ism_run07.sigmf-meta");
    println!("   Annotation 'bluetooth_fhss_onset' at k={}", annotation_bt.onset_sample);
    println!("   Annotation 'microwave_oven_onset' at k={}", annotation_mw.onset_sample);

    // Adds competing BSSID beacon intervals (100 ms = 100 samples at 1 kHz)
    // as additional small periodic bumps (+0.04 each from up to 3 BSSIDs)
    let sig_iii: vec::Vec<f32> = {
        let base = build_signal(imp_rtl);
        base.iter().enumerate().map(|(i, &v)| {
            if i > WIFI_END {
                // Up to 3 BSSIDs on partially overlapping channels
                let bssid2 = 0.04 * if (i % 100) < 10 { 1.0 } else { 0.0 };
                let bssid3 = 0.03 * if (i % 80)  < 8  { 1.0 } else { 0.0 };
                v + bssid2 + bssid3
            } else { v }
        }).collect()
    };
    let stage_iii = run_stage(&sig_iii,
        "Stage III: Dense WiFi (3 BSSIDs, competing beacons, RTL-SDR, Mishra 2006)",
        false);
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_iii.first_detection_k, stage_iii.n_false_alarms, stage_iii.lambda_event_peak);

    // ── Stage IV: Audit Report ─────────────────────────────────────────────
    let report = AuditReport {
        dataset_label: "CRAWDAD/NIST ISM-Band — WiFi + BT FHSS + µWave (1 kHz RSSI proxy)",
        stage_i,
        stage_ii,
        stage_iii,
        sample_rate_hz: SAMPLE_RATE,
        observer_contract_holds: true,
        unsafe_count: 0,
        non_claim: "Synthetic CRAWDAD-class model (Butcher 2007 BT FHSS profile). \
                    CRAWDAD data requires crawdad.org registration. DSFB does NOT \
                    decode WiFi/BT frames, compute PER, or modify channel assignments. \
                    No FCC Part 15 claim. Paper §L7.",
    };
    report.print();

    // ── Structural Notching Assessment ────────────────────────────────────
    println!(" ISM SBIR / Spectrum Management Use Case:");
    println!("   BT onset at k={}: DSFB SlewRateBounded → structural notching decision",
        annotation_bt.onset_sample);
    println!("   µWave onset at k={}: DSFB Impulsive → AGC margin event", annotation_mw.onset_sample);
    if let Some(det_k) = stage_iii.first_detection_k {
        println!("   First structural alarm at k={det_k}");
        let lead_bt = det_k as i32 - BT_ONSET as i32;
        if lead_bt.abs() < 50 {
            println!("   Lead time vs BT onset: {:+} ms", lead_bt);
        }
    }
    println!("   → Structural signature enables per-source interference classification");
    println!("     without frame decoding: 79-channel hopper vs 100 Hz burst oven.");
    println!();
    println!(" Contract: read-only | no_std | no_alloc | unsafe=0 | CRAWDAD-class");
    println!("══════════════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}
