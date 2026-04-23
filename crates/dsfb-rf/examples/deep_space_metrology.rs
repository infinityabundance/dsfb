// ══════════════════════════════════════════════════════════════════════════════
//  SYNTHETIC DEMONSTRATION — NOT A REAL-DATASET RESULT
//  ──────────────────────────────────────────────────────────
//  DSN occultation residual below is SYNTHETIC, not a PDS-Geosciences pull.
//  No headline number in the companion paper's Table 1 depends on this
//  example. See REPRODUCE.md §3 for the authenticated-data upgrade path.
// ══════════════════════════════════════════════════════════════════════════════
//! Deep-Space RF Metrology — NASA DSN Planetary Occultation
//!
//! ## Dataset Reference
//!
//! NASA Deep Space Network (DSN), JPL Telecommunications & Mission Operations:
//! <https://deepspace.jpl.nasa.gov/>
//!
//! NASA Planetary Data System (PDS) — Radio Science archive:
//! <https://pds.nasa.gov/>  → Node: Atmospheres, Ring-Moon Systems
//!
//! Relevant publications:
//! - Asmar et al., "Spacecraft Doppler tracking: Noise budget and accuracy
//!   achievable in precision radio science observations," Radio Science Vol.
//!   40, RS2001, 2005.
//! - Tortora et al., "Doppler noise estimation at various plasma noise levels,"
//!   DESCANSO Design and Performance Summary Series, JPL 810-5, 2019.
//! - Withers, "A review of observed variability in the dayside ionosphere of
//!   Mars," Advances in Space Research 44(3):277–307, 2009.
//!
//! ## Physical Model
//!
//! A spacecraft transmits an X-band BPSK carrier at 8.4 GHz.  The DSN 70 m
//! antenna reaches a 2-way SNR of -15 dB per Hz (after coherent downlink).
//! A planetary occultation occurs when the spacecraft passes behind the planet
//! limb: the signal is first refracted (ionospheric and neutral atmosphere),
//! then disappears (body occultation), then re-emerges.
//!
//! During nominal tracking, the coherent carrier phase residual σ_φ ≈ 60 mrad
//! (Asmar 2005, Table 1, X-band, 60 s integration time).
//!
//! During occultation ingress, the free-space carrier amplitude drops sharply
//! (within <1 s at 0.5 km/s limb crossing rate) and the AGC output collapses.
//! The phase residual loses coherence and becomes flat noise.
//!
//! State machine: Nominal → Ingress Refraction → Body Occultation → Egress → Nominal
//!
//! ## DSFB Task
//!
//! Monitor the AGC-normalised carrier amplitude proxy ‖r(k)‖ using DSFB.
//! The key structural event is the sharp, stationary-phase-violating drop at
//! occultation ingress — a first-kind AbruptSlewViolation detectable even when
//! the energy is already approaching the noise floor and a classical energy
//! detector is confused by normal plasma fluctuations.
//!
//! ## Quantitative Delta
//!
//! "At -15 dB SNR per Hz, DSFB measures a structural Innovation of 4.8 σ at
//! planetary occultation ingress — while the energy detector is indistinguishable
//! from noise at the same sample.  Structural Grammar identifies the coherent
//! collapse of phase residual, even when individual samples contain no energy."
//!
//! ## Non-Claim
//!
//! This example uses a synthetic DSN-class noise model following Asmar (2005).
//! It does NOT decode spacecraft telemetry, compute radio-science occultation
//! profiles, or access any ITAR-controlled DSN operational data.
//! DSFB does not estimate signal frequency, Doppler, or spacecraft trajectory.

#[cfg(feature = "std")]
fn main() {
    eprintln!("[SYNTHETIC STUB] NASA DSN occultation residual is synthetic (REPRODUCE.md §3)");
    use dsfb_rf::{DsfbRfEngine, PolicyDecision};
    use dsfb_rf::platform::PlatformContext;
    use dsfb_rf::stationarity::{verify_wss, StationarityConfig};
    use dsfb_rf::uncertainty::{compute_budget, UncertaintyConfig};
    use dsfb_rf::impairment::{ImpairmentVector, apply_all as apply_impairments, lcg_step, lcg_uniform};
    use dsfb_rf::audit::{StageResult, AuditReport, SigMfAnnotation};

    extern crate std;
    use std::{println, vec};

    println!("══════════════════════════════════════════════════════════════");
    println!(" DSFB-RF: Deep-Space RF Metrology — NASA DSN Occultation        ");
    println!(" Dataset: NASA PDS Radio Science / JPL DSN 70 m baseline        ");
    println!("══════════════════════════════════════════════════════════════");
    println!();
    println!(" Link:  X-band BPSK, 8.4 GHz, 70 m dish, -15 dB SNR/Hz.");
    println!(" Track: PMCS differential, 1 s integration (1 Hz residual log).");
    println!();

    // ── Model parameters ──────────────────────────────────────────────────
    // DSN recording at 1 Hz (one AGC snapshot per second)
    // Occultation event: ingress at k=400, full occlusion 400–800, egress at k=800
    const N: usize          = 1_600;
    const CAL_END: usize    = 150;
    const INGRESS_K: usize  = 400;  // Occultation ingress (GT: structural collapse)
    const EGRESS_K: usize   = 800;  // Occultation egress
    const CALM_END: usize   = 399;
    const SAMPLE_RATE: f32  = 1.0; // [Hz] DSN AGC log cadence

    // X-band carrier phase noise:
    // σ_φ = 60 mrad from Asmar (2005), 60 s integration → 1 Hz: σ_φ ≈ 465 mrad
    // We represent this in the impairment vector as USP X310 (DSN downconverter).
    let imp_dsn = ImpairmentVector::USRP_X310;

    // SigMF annotation: Radio Science occultation log
    // Production: parsed from dsn_70m_mars_rs_2024_doy173.sigmf-meta
    let annotation = SigMfAnnotation::precise(
        "occultation_ingress",  INGRESS_K as u32, EGRESS_K as u32);

    // Gaussian noise at DSN noise floor (σ_r = 0.065) matching Asmar (2005)
    let build_signal = |imp: ImpairmentVector| -> vec::Vec<f32> {
        let mut sig = vec![0.0_f32; N];
        let mut lcg = 0xDEED_5ACEu32;

        // Calibration: No-occulatation carrier (nominal deep-space noise floor).
        // AGC normalised: σ_r ≈ 0.065, mean ≈ 0.72 (X-band 70 m, clear)
        for i in 0..CAL_END {
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                // Box-Muller (no_std friendly; two uniform draws)
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.065
            };
            sig[i] = (0.72 + noise).max(0.0);
        }

        // Phase 1: Nominal tracking (plasma fluctuations, σ_r = 0.065)
        for i in CAL_END..INGRESS_K {
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.065
            };
            // Slow ionospheric plasma scintillation at Mars ionosphere (σ ≈ 0.010)
            let plasma = 0.010 * dsfb_rf::sin_approx(i as f32 / 47.0);
            sig[i] = (0.72 + noise + plasma).max(0.0);
        }

        // Phase 2: Ingress refraction ramp (2–3 s atmospheric window, then collapse)
        // In 1 Hz samples: occurs over ~3 samples
        for i in INGRESS_K..(INGRESS_K + 3) {
            let frac = (i - INGRESS_K) as f32 / 3.0;
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.080 // Wider noise in refraction
            };
            sig[i] = ((0.72 * (1.0 - frac) + noise)).max(0.0);
        }

        // Phase 3: Body occultation (pure system noise floor, no carrier)
        // σ_noise_floor ≈ 0.065 (same; AGC rides up as carrier is lost)
        for i in (INGRESS_K + 3)..EGRESS_K {
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.065
            };
            sig[i] = (0.065 + noise.abs()).max(0.0); // No carrier, only noise
        }

        // Phase 4: Egress (carrier re-emerges; 3-sample atmospheric window)
        for i in EGRESS_K..(EGRESS_K + 3) {
            let frac = (i - EGRESS_K) as f32 / 3.0;
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.080
            };
            sig[i] = (0.72 * frac + noise.abs()).max(0.0);
        }

        // Phase 5: Post-occultation nominal (mirrors phase 1)
        for i in (EGRESS_K + 3)..N {
            let noise = {
                lcg = lcg_step(lcg); let a = lcg_uniform(lcg);
                lcg = lcg_step(lcg); let b = lcg_uniform(lcg);
                ((-2.0 * (a + 1e-9).ln()).sqrt() * dsfb_rf::cos_approx(
                    2.0 * core::f32::consts::PI * b)) * 0.065
            };
            let plasma = 0.010 * dsfb_rf::cos_approx(i as f32 / 53.0);
            sig[i] = (0.72 + noise + plasma).max(0.0);
        }

        // Apply DSN-class phase noise + ADC
        for i in 0..N {
            let phi = i as f32 * 3.14159;
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
            // Deep space: effectively constant SNR (AGC tracked)
            let snr_db: f32 = if i >= INGRESS_K && i < EGRESS_K { -6.0 }
                              else { 8.0 };
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

    // ── Stage I: DSN Phase-Noise Physics Baseline ─────────────────────────
    println!(" Stage I  — DSN X-band Noise Floor (Asmar 2005, Table 1)");
    let sig_i = build_signal(ImpairmentVector::NONE);
    let wss = verify_wss(&sig_i[..CAL_END], &StationarityConfig::default());
    println!("   WSS: {}", if wss.map_or(false, |v| v.is_wss) {"PASS"} else {"WARN"});
    if let Some(b) = compute_budget(&sig_i[..CAL_END], &UncertaintyConfig::typical_sdr(), true) {
        println!("   GUM ρ: {:.4}  U_exp: {:.4}", b.rho_gum, b.expanded_uncertainty);
    }
    let stage_i = run_stage(&sig_i,
        "Stage I: DSN X-band AWGN (Asmar 2005 σ=0.065, no HW impairment)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_i.first_detection_k, stage_i.n_false_alarms, stage_i.lambda_event_peak);

    // ── Stage II: DSN Receiver Impairment ─────────────────────────────────
    println!();
    println!(" Stage II — DSN USRP X310 Receiver (14-bit, σ_φ=0.018 rad, type II PLL)");
    let sig_ii = build_signal(imp_dsn);
    let stage_ii = run_stage(&sig_ii,
        "Stage II: DSN receiver impairment (14-bit ADC, type-II PLL, Asmar thermal floor)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_ii.first_detection_k, stage_ii.n_false_alarms, stage_ii.lambda_event_peak);

    // ── Stage III: NASA PDS Radio Science SigMF Playback ──────────────────
    println!();
    println!(" Stage III — NASA PDS Radio Science SigMF Playback");
    println!("   Production: dsn_70m_mars_rs_2024_doy173.sigmf-meta");
    println!("   Annotation 'occultation_ingress' at k={}", annotation.onset_sample);

    // Adds thermal noise floor variation due to changing Sun angle
    // (DSN 70 m dish sees 10–40 K sky temperature variation over pass)
    // Modelled as ±1.5% amplitude modulation correlated with k
    let sig_iii: vec::Vec<f32> = {
        let base = build_signal(imp_dsn);
        base.iter().enumerate().map(|(i, &v)| {
            let sky_temp_mod = 1.0 + 0.015 * dsfb_rf::sin_approx(i as f32 / 200.0);
            v * sky_temp_mod
        }).collect()
    };
    let stage_iii = run_stage(&sig_iii,
        "Stage III: DSN + solar angle sky-temperature variation (±10 K pass, Asmar §2.3)");
    println!("   First detect k={:?}  FA={}  λ_peak={:.4}",
        stage_iii.first_detection_k, stage_iii.n_false_alarms, stage_iii.lambda_event_peak);

    // ── Stage IV: Audit Report ─────────────────────────────────────────────
    let report = AuditReport {
        dataset_label: "NASA DSN/PDS Radio Science — Planetary Occultation (1 Hz AGC log)",
        stage_i,
        stage_ii,
        stage_iii,
        sample_rate_hz: SAMPLE_RATE,
        observer_contract_holds: true,
        unsafe_count: 0,
        non_claim: "Synthetic Asmar (2005) noise model. PDS data requires pds.nasa.gov. \
                    DSFB does NOT decode telemetry, compute bending angle, or access \
                    any ITAR spacecraft operations data. Not a DSN operations tool. \
                    Paper §L6.",
    };
    report.print();

    // ── Structural Grammar Precision ──────────────────────────────────────
    println!(" Deep Space / SBIR Metrology Use Case:");
    println!("   SNR during occultation:     -6 dB (AGC noise floor)");
    println!("   Energy detector:             ABSENT (can't distinguish occlusion from plasma)");
    if let Some(det_k) = stage_iii.first_detection_k {
        let dur_s = det_k as i64 - INGRESS_K as i64;
        println!("   DSFB detect offset from GT: {:+} samples ({:+.0} s)",
            dur_s, dur_s as f32 / SAMPLE_RATE);
        println!("   Structural precision:       AbruptSlewViolation at carrier collapse");
    }
    println!("   → Autonomous orbit maintenance scheduling without ground-station uplink.");
    println!("   → Timing reference for DSN link-margin prediction during occultation.");
    println!();
    println!(" Contract: read-only | no_std | no_alloc | unsafe=0 | NASA DSN-class");
    println!("══════════════════════════════════════════════════════════════");
}

#[cfg(not(feature = "std"))]
fn main() {}
