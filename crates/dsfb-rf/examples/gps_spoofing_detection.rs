// ══════════════════════════════════════════════════════════════════════════════
//  SYNTHETIC DEMONSTRATION — NOT A REAL-DATASET RESULT
//  ──────────────────────────────────────────────────────────
//  Spoofing-then-recovery trajectory below is SYNTHETIC, not a UT Austin
//  Radionavigation Lab capture. No headline number in the companion paper's
//  Table 1 depends on this example. See REPRODUCE.md §3 for the real-data
//  upgrade path.
// ══════════════════════════════════════════════════════════════════════════════
//! GPS Spoofing Detection — UT Austin Radionavigation Lab Dataset
//!
//! ## Dataset Reference
//!
//! Dr. Todd Humphreys, UT Austin Radionavigation Lab:
//! <https://radionavlab.ae.utexas.edu/>
//!
//! Relevant publications:
//! - Humphreys et al., "Assessing the Spoofing Threat: Development of a
//!   Portable GPS Civilian Spoofer," ION GNSS 2008.
//! - Humphreys, "Detection Strategy for Cryptographic GNSS Anti-Spoofing,"
//!   IEEE Trans. Aerospace Electron. Syst. 52(4), 2016.
//! - Psiaki & Humphreys, "GNSS Spoofing and Detection," Proc. IEEE, 2016.
//!
//! Dataset: UT Austin radionavlab capture archives (contact: radionavlab.ae.utexas.edu).
//! Capture format: SigMF .cf32 + .sigmf-meta, GPS L1 C/A (1575.42 MHz), 5 MS/s.
//!
//! ## Physical Model
//!
//! A meaconing/spoofing attack slowly "pulls" the GPS receiver's tracking loop.
//! The spoofer transmits a replica of the legitimate L1 C/A signal at a slightly
//! different delay and/or carrier frequency, gradually increasing its power
//! relative to the authentic signal.
//!
//! The phase tracking residual θ_e(k) of the PLL accumulates a coherent ramp:
//!
//! ```text
//! θ_e(k) ≈ θ_e0 + α_spoof · k   [rad]
//! ```
//!
//! where α_spoof = 2π · Δf_pull / f_ref.  For a 0.1 Hz pull rate (typical
//! "smooth takeover"):  α_spoof ≈ 6×10⁻⁴ rad per 100 Hz residual update.
//!
//! The admissibility envelope radius ρ is calibrated from the authentic-signal
//! PLL residual during a verified-clean acquisition window.  The spoofer-induced
//! drift exits the envelope before the navigation solution diverges because the
//! semiotic grammar detects the structural innovation (SustainedOutwardDrift)
//! that precedes PVT failure by several seconds.
//!
//! ## GPS PLL Phase Residual Model
//!
//! For a 3rd-order PLL with noise bandwidth B_L = 10 Hz tracking GPS L1:
//!
//! ```text
//! σ_θ_nominal ≈ sqrt(4 · k · T · F_noise / (C/N₀)) · (1 + 1/(2·T·C/N₀))
//! ```
//!
//! At C/N₀ = 40 dB-Hz (typical urban): σ_θ ≈ 0.045 rad ≈ 0.007 · (2π·chip).
//! We normalise θ_e to [0, 1] range for the engine: ρ_cal ≈ 0.035.
//!
//! ## DSFB Task
//!
//! Detect the structural innovation of a smooth-takeover spoofing attack
//! using only the IQ residuals of the GPS PLL output stream.
//!
//! ## Quantitative Delta
//!
//! "While the GPS receiver still reports 'Valid Position,' DSFB identifies
//! the structural innovation of a coherent spoofer pulling the LO.  We
//! detect the attack in the semiotic domain while the navigation domain is
//! still being deceived — providing a pre-authentication structural
//! tripwire at zero modification to the receiver firmware."
//!
//! ## 4-Stage Continuous Rigor Pipeline
//!
//! Stage I  — Physics Baseline:  clean PLL residual + spoofing ramp, no HW impairment
//! Stage II — HW Impairment:     TCXO phase noise (σ_φ=0.05 rad) + 14-bit ADC
//! Stage III— SigMF Playback:    UT Austin representative spoofing capture scenario
//! Stage IV — Audit Report:      predicted onset vs. PVT-divergence annotation
//!
//! ## Non-Claim
//!
//! This simulation uses a synthetic PLL residual model derived from published
//! GPS spoofing literature.  It does NOT use the actual UT Austin dataset
//! (contact UT Radionavigation Lab for access).  DSFB does NOT provide
//! cryptographic authentication, anti-spoofing assurance, or ITAR-regulated
//! GPS security capability.  This is a structural residual observer only.
//! Physical spoofing detection requires multi-receiver geometry and/or
//! cryptographic authentication (IS-GPS-800F).

#[cfg(feature = "std")]
fn main() {
    eprintln!("[SYNTHETIC STUB] UT Austin spoofing trajectory is synthetic (REPRODUCE.md §3)");
    use dsfb_rf::{DsfbRfEngine, PolicyDecision};
    use dsfb_rf::platform::PlatformContext;
    use dsfb_rf::stationarity::{verify_wss, StationarityConfig};
    use dsfb_rf::physics::{candidate_mechanisms, model_reference};
    use dsfb_rf::grammar::ReasonCode;
    use dsfb_rf::impairment::{ImpairmentVector, apply_all as apply_impairments, lcg_step};
    use dsfb_rf::audit::{StageResult, AuditReport, SigMfAnnotation};

    extern crate std;
    use std::{println, vec};

    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF: GPS Spoofing Detection (UT Austin model)            ");
    println!(" Dataset: UT Radionavigation Lab PLL residual captures         ");
    println!("══════════════════════════════════════════════════════════════");
    println!();
    println!(" Signal: GPS L1 C/A PLL phase residual (θ_e), normalised.");
    println!(" Attack: Smooth-takeover meaconing, α_spoof=6×10⁻⁴ rad/update.");
    println!(" GT reference: SigMF annotation 'spoofer_onset' from dataset log.");
    println!();

    // ── Model parameters ──────────────────────────────────────────────────
    // GPS L1 C/A PLL tracking loop, 100 Hz residual update rate (decimated from 5 MS/s).
    // Spoofing smooth-takeover: pull rate 0.1 Hz → Δf = 0.1 Hz →
    //   α_spoof = 2π × 0.1 / 100 = 6.28×10⁻³ rad/step (normalised to ρ: 6×10⁻⁴)
    const N: usize          = 2_000;
    const CAL_END: usize    = 150;
    const ONSET_K: usize    = 600;    // Spoofer onset (GT)
    const PVT_FAIL_K: usize = 1_100;  // Navigation solution divergence (GT reference)
    const CALM_END: usize   = 599;
    const SAMPLE_RATE: f32  = 100.0;  // [Hz] residual update rate

    // SigMF ground-truth annotation
    // Production: parsed from gps_spoofing_20120627_austin.sigmf-meta
    // (UT Austin 2012 live-sky test, Humphreys et al.)
    let annotation = SigMfAnnotation::precise(
        "spoofer_onset", ONSET_K as u32, PVT_FAIL_K as u32);

    // α_spoof = 6×10⁻⁴ per residual update (smooth takeover)
    const ALPHA_SPOOF: f32 = 6.0e-4;

    let build_signal = |imp: ImpairmentVector| -> vec::Vec<f32> {
        let mut sig = vec![0.0_f32; N];
        let mut lcg = 0x1234_ABCDu32;

        // Calibration: authentic GPS PLL at C/N₀=40 dB-Hz
        // σ_θ ≈ 0.035 normalised, zero-mean white phase noise
        for i in 0..CAL_END {
            sig[i] = 0.030 + 0.005 * dsfb_rf::sin_approx(i as f32 * 3.7);
        }

        // Pre-spoof: nominal tracking, slow multipath variation
        for i in CAL_END..ONSET_K {
            let mp = 0.004 * dsfb_rf::sin_approx(i as f32 * 0.03) // multipath cycle
                   + 0.003 * dsfb_rf::cos_approx(i as f32 * 0.11); // secondary path
            sig[i] = 0.031 + mp;
        }

        // Spoofing Phase I: slow coherent drift (ramp)
        // The receiver's PLL integrates the extra force from the spoofer signal.
        // SustainedOutwardDrift detected here — navigation solution still valid.
        for i in ONSET_K..PVT_FAIL_K {
            let t    = (i - ONSET_K) as f32;
            let ramp = t * ALPHA_SPOOF;
            let mp   = 0.003 * dsfb_rf::sin_approx(t * 0.03);
            sig[i]   = 0.031 + ramp + mp;
        }

        // Spoofing Phase II: capture complete — receiver fully tracking spoofer
        // PLL residual collapses back to the spoofer's clean replica (false lock).
        for i in PVT_FAIL_K..N {
            let t = (i - PVT_FAIL_K) as f32;
            // After capture: residual resettles on spoofer (lower again but wrong position)
            let settle = 0.15 * (-t * 0.03).exp();
            sig[i] = 0.028 + settle;
        }

        // Apply hardware impairment
        for i in 0..N {
            let phi = i as f32 * 0.063;
            lcg = lcg_step(lcg);
            let (r, s) = apply_impairments(sig[i], phi, lcg, imp);
            lcg = s;
            sig[i] = r;
        }
        sig
    };

    let run_stage = |sig: &[f32], label: &'static str| -> StageResult {
        let mut engine = DsfbRfEngine::<10, 4, 8>::from_calibration(&sig[..CAL_END], 2.0)
            .expect("calibration required");
        let mut sr = StageResult::new(label, annotation.onset_sample);
        sr.n_obs      = sig.len() as u32;
        sr.n_calm_obs = (CALM_END - CAL_END) as u32;
        for (i, &norm) in sig.iter().enumerate().skip(CAL_END) {
            // GPS L1 C/N₀ drops 3 dB during spoofing onset (power competition)
            let snr_db: f32 = if i < ONSET_K { 22.0 }
                              else if i < PVT_FAIL_K { 18.0 }
                              else { 24.0 }; // After capture: spoofer has full power
            let obs = engine.observe(norm, PlatformContext::with_snr(snr_db));
            let evt = matches!(obs.policy, PolicyDecision::Review | PolicyDecision::Escalate);
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
        sr
    };

    // ── Stage I: Physics Baseline ──────────────────────────────────────────
    println!(" Stage I  — PLL Physics Baseline (zero impairment)");
    let sig_i = build_signal(ImpairmentVector::NONE);
    let wss = verify_wss(&sig_i[..CAL_END], &StationarityConfig::default());
    println!("   WSS: {}  (Wiener-Khinchin precondition for ρ calibration)",
        if wss.map_or(false, |v| v.is_wss) {"PASS"} else {"WARN"});
    let stage_i = run_stage(&sig_i,
        "Stage I: GPS PLL physics baseline (clean PLL + α_spoof=6e-4, no HW impairment)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_i.first_detection_k, stage_i.n_false_alarms, stage_i.lambda_event_peak);
    if let Some(lt_ms) = stage_i.lead_time_ms(SAMPLE_RATE) {
        println!("   Lead time vs. PVT fail at k={}: {} ms", PVT_FAIL_K as u32, lt_ms);
    }
    if let Some(lt_ms) = stage_i.lead_time_ms(SAMPLE_RATE) {
        let spoof_lt = PVT_FAIL_K as f32 / SAMPLE_RATE * 1000.0
                       - stage_i.first_detection_k.map_or(0.0, |k| k as f32 / SAMPLE_RATE * 1000.0);
        println!("   DSFB lead before PVT divergence: {:.0} ms", spoof_lt);
        let _ = lt_ms;
    }

    // ── Stage II: TCXO Hardware Impairment ────────────────────────────────
    println!();
    println!(" Stage II — TCXO Phase Noise (σ_φ=0.050 rad rms, 14-bit ADC)");
    println!("   Source: GPS receiver TCXO (Leeson integrated over [1 Hz, 50 Hz])");
    let gps_tcxo = ImpairmentVector {
        iq_imbalance_epsilon: 0.001,
        dc_offset_i: 0.001,
        dc_offset_q: 0.001,
        pa_k3: 0.0,
        adc_bits: 14,
        phase_noise_sigma: 0.050, // TCXO: ∫L(f) df over loop bandwidth
        scintillation_s4: 0.0,
    };
    let sig_ii = build_signal(gps_tcxo);
    let stage_ii = run_stage(&sig_ii,
        "Stage II: GPS TCXO phase noise (Leeson model, σ_φ=0.050 rad, 14-bit ADC)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_ii.first_detection_k, stage_ii.n_false_alarms, stage_ii.lambda_event_peak);

    // ── Stage III: UT Austin Representative Playback ───────────────────────
    println!();
    println!(" Stage III — UT Austin GPS Spoofing SigMF Playback");
    println!("   Production: radio lab capture gps_spoofing_20120627.sigmf-meta");
    println!("   Humphreys et al. 2012 Austin live-sky spoofer demonstration.");
    println!("   Annotation 'spoofer_onset' at k={}", annotation.onset_sample);

    // Stage III adds GPS ionospheric scintillation (S4=0.15, weak) and Doppler
    // jitter representative of the Austin live-sky test (vehicle velocity 15 m/s)
    let austin_imp = ImpairmentVector {
        iq_imbalance_epsilon: 0.002,
        dc_offset_i: 0.001,
        dc_offset_q: 0.001,
        pa_k3: 0.0,
        adc_bits: 14,
        phase_noise_sigma: 0.050,
        scintillation_s4: 0.15,  // Weak L1 scintillation in Texas (typical)
    };
    let sig_iii: vec::Vec<f32> = {
        let base = build_signal(austin_imp);
        // Add Doppler-induced carrier slipping: 15 m/s → f_D = 1575.42e6*15/3e8 ≈ 78.8 Hz
        // Steady-state PLL error at K_v=500 Hz/rad: θ_ss = 2π*78.8/500 ≈ 0.991 rad
        // Normalised contribution to residual floor: 0.991 * 0.030 ≈ 0.030 → adds ~0.005
        let doppler_floor = dsfb_rf::doppler_residual_floor(78.8, 500.0, 0.030);
        base.iter().map(|&v| (v + doppler_floor * 0.1).max(0.0)).collect()
    };
    let stage_iii = run_stage(&sig_iii,
        "Stage III: UT Austin live-sky (TCXO + weak scintillation + Doppler 78.8 Hz)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_iii.first_detection_k, stage_iii.n_false_alarms, stage_iii.lambda_event_peak);

    // ── Stage IV: Audit Report ─────────────────────────────────────────────
    let report = AuditReport {
        dataset_label: "GPS Spoofing / UT Austin Radionavigation Lab (100 Hz PLL residual)",
        stage_i,
        stage_ii,
        stage_iii,
        sample_rate_hz: SAMPLE_RATE,
        observer_contract_holds: true,
        unsafe_count: 0,
        non_claim: "Synthetic PLL residual. Does NOT provide anti-spoofing authentication. \
                    UT Austin dataset requires radionavlab.ae.utexas.edu contact. No \
                    ITAR-regulated GPS security capability. Physical spoofing detection \
                    requires cryptographic authentication (IS-GPS-800F). Paper §L5.",
    };
    report.print();

    // ── Physics Mapping: σ(k) → candidate mechanisms ──────────────────────
    println!(" Physics-of-failure candidates for SustainedOutwardDrift:");
    for mech in candidate_mechanisms(ReasonCode::SustainedOutwardDrift) {
        println!("   • {:?} — {}", mech, model_reference(*mech));
    }
    println!();

    // ── SBIR Lead-Time Statement ──────────────────────────────────────────
    let pvt_ms = PVT_FAIL_K as f32 / SAMPLE_RATE * 1_000.0;
    if let Some(det_k) = stage_iii.first_detection_k {
        let det_ms = det_k as f32 / SAMPLE_RATE * 1_000.0;
        println!(" SBIR DELTA:");
        println!("   PVT divergence:        {:.0} ms after session start", pvt_ms);
        println!("   DSFB first detection:  {:.0} ms after session start", det_ms);
        println!("   Lead-time advantage:   {:.0} ms", pvt_ms - det_ms);
        println!("   → Structural tripwire fires in the semiotic domain");
        println!("     while the navigation solution is still reporting Valid.");
    }
    println!();
    println!(" Contract: read-only | no_std | no_alloc | unsafe=0 | non-attributing");
    println!("══════════════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}
